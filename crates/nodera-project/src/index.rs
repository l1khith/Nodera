use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

use crate::error::{ProjectError, Result};
use crate::graph::{ProjectNode, ProjectNodeKind};

/// Persistent SQLite-backed project index located at `.nodera/index/project.db`.
pub struct ProjectIndex {
    db_path: PathBuf,
}

impl ProjectIndex {
    /// Opens or initializes the SQLite database at `.nodera/index/project.db`.
    pub fn open_or_create(index_dir: &Path) -> Result<Self> {
        fs::create_dir_all(index_dir).map_err(|e| ProjectError::Io {
            path: index_dir.to_path_buf(),
            source: e,
        })?;

        let db_path = index_dir.join("project.db");
        let conn = Connection::open(&db_path).map_err(|e| ProjectError::Database {
            reason: format!("Failed to open index SQLite database: {e}"),
        })?;

        // Initialize schema
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;

            CREATE TABLE IF NOT EXISTS source_files (
                path TEXT PRIMARY KEY,
                package TEXT NOT NULL,
                mtime INTEGER NOT NULL,
                size INTEGER NOT NULL,
                hash TEXT NOT NULL,
                sloc INTEGER NOT NULL,
                symbol_count INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS symbols (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                visibility TEXT NOT NULL,
                start_line INTEGER,
                end_line INTEGER,
                signature TEXT,
                doc TEXT,
                parent_id TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_path);
            CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
            CREATE INDEX IF NOT EXISTS idx_symbols_kind ON symbols(kind);

            CREATE TABLE IF NOT EXISTS symbol_references (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                caller_id TEXT NOT NULL,
                target TEXT NOT NULL,
                kind TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_refs_caller ON symbol_references(caller_id);
            CREATE INDEX IF NOT EXISTS idx_refs_target ON symbol_references(target);
            "#,
        )
        .map_err(|e| ProjectError::Database {
            reason: format!("Failed to initialize index schema: {e}"),
        })?;

        Ok(Self { db_path })
    }

    fn connect(&self) -> Result<Connection> {
        Connection::open(&self.db_path).map_err(|e| ProjectError::Database {
            reason: format!("Failed to connect to index DB: {e}"),
        })
    }

    /// Indexes or updates a source file entry.
    #[allow(clippy::too_many_arguments)]
    pub fn index_file(
        &self,
        rel_path: &str,
        package: &str,
        mtime: u64,
        size: u64,
        hash: &str,
        sloc: usize,
        symbol_count: usize,
    ) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT OR REPLACE INTO source_files (path, package, mtime, size, hash, sloc, symbol_count)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![rel_path, package, mtime as i64, size as i64, hash, sloc as i64, symbol_count as i64],
        )
        .map_err(|e| ProjectError::Database {
            reason: format!("Failed to index file {rel_path}: {e}"),
        })?;
        Ok(())
    }

    /// Removes a source file and cascades deletion to all its symbols and references.
    pub fn remove_file(&self, rel_path: &str) -> Result<()> {
        let mut conn = self.connect()?;
        let tx = conn.transaction().map_err(|e| ProjectError::Database {
            reason: format!("Failed to start transaction: {e}"),
        })?;

        tx.execute(
            "DELETE FROM symbol_references WHERE file_path = ?1",
            params![rel_path],
        )
        .map_err(|e| ProjectError::Database {
            reason: format!("Failed to delete references for {rel_path}: {e}"),
        })?;

        tx.execute(
            "DELETE FROM symbols WHERE file_path = ?1",
            params![rel_path],
        )
        .map_err(|e| ProjectError::Database {
            reason: format!("Failed to delete symbols for {rel_path}: {e}"),
        })?;

        tx.execute(
            "DELETE FROM source_files WHERE path = ?1",
            params![rel_path],
        )
        .map_err(|e| ProjectError::Database {
            reason: format!("Failed to delete file entry for {rel_path}: {e}"),
        })?;

        tx.commit().map_err(|e| ProjectError::Database {
            reason: format!("Failed to commit file removal transaction: {e}"),
        })?;

        Ok(())
    }

    /// Inserts a symbol into the index.
    pub fn insert_symbol(&self, node: &ProjectNode) -> Result<()> {
        if node.kind == ProjectNodeKind::File {
            return Ok(());
        }

        let conn = self.connect()?;
        let (start_line, end_line) = match node.span {
            Some(s) => (Some(s.start_line as i64), Some(s.end_line as i64)),
            None => (None, None),
        };

        conn.execute(
            r#"
            INSERT OR REPLACE INTO symbols (id, file_path, name, kind, visibility, start_line, end_line, signature, doc, parent_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                node.id,
                node.file,
                node.name,
                node.kind.to_string(),
                node.visibility,
                start_line,
                end_line,
                node.signature,
                node.doc,
                node.parent_id,
            ],
        )
        .map_err(|e| ProjectError::Database {
            reason: format!("Failed to insert symbol {}: {e}", node.id),
        })?;

        Ok(())
    }

    /// Inserts a reference into the index.
    pub fn insert_reference(
        &self,
        caller_id: &str,
        target: &str,
        kind: &str,
        file_path: &str,
        line: usize,
    ) -> Result<()> {
        let conn = self.connect()?;
        conn.execute(
            r#"
            INSERT INTO symbol_references (caller_id, target, kind, file_path, line)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![caller_id, target, kind, file_path, line as i64],
        )
        .map_err(|e| ProjectError::Database {
            reason: format!("Failed to insert reference from {caller_id} to {target}: {e}"),
        })?;
        Ok(())
    }

    /// Returns the total count of indexed symbols.
    pub fn symbol_count(&self) -> Result<usize> {
        let conn = self.connect()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM symbols WHERE kind != 'file'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| ProjectError::Database {
                reason: format!("Failed to count symbols: {e}"),
            })?;
        Ok(count as usize)
    }

    /// Returns the total count of indexed files.
    pub fn file_count(&self) -> Result<usize> {
        let conn = self.connect()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM source_files", [], |row| row.get(0))
            .map_err(|e| ProjectError::Database {
                reason: format!("Failed to count source files: {e}"),
            })?;
        Ok(count as usize)
    }

    /// Searches symbols by name prefix or substring match.
    pub fn search_symbols(&self, query: &str, limit: usize) -> Result<Vec<ProjectNode>> {
        let conn = self.connect()?;
        let pattern = format!("%{}%", query.trim());
        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, file_path, name, kind, visibility, start_line, end_line, signature, doc, parent_id
                FROM symbols
                WHERE name LIKE ?1 OR signature LIKE ?1
                ORDER BY length(name) ASC, name ASC
                LIMIT ?2
                "#,
            )
            .map_err(|e| ProjectError::Database {
                reason: format!("Failed to prepare search statement: {e}"),
            })?;

        let rows = stmt
            .query_map(params![pattern, limit as i64], |row| {
                let id: String = row.get(0)?;
                let file: String = row.get(1)?;
                let name: String = row.get(2)?;
                let kind_str: String = row.get(3)?;
                let visibility: String = row.get(4)?;
                let start_line: Option<usize> = row.get::<_, Option<i64>>(5)?.map(|v| v as usize);
                let end_line: Option<usize> = row.get::<_, Option<i64>>(6)?.map(|v| v as usize);
                let signature: Option<String> = row.get(7)?;
                let doc: Option<String> = row.get(8)?;
                let parent_id: Option<String> = row.get(9)?;

                let kind = match kind_str.as_str() {
                    "file" => ProjectNodeKind::File,
                    "module" => ProjectNodeKind::Module,
                    "struct" => ProjectNodeKind::Struct,
                    "enum" => ProjectNodeKind::Enum,
                    "variant" => ProjectNodeKind::EnumVariant,
                    "trait" => ProjectNodeKind::Trait,
                    "impl" => ProjectNodeKind::Implementation,
                    "function" => ProjectNodeKind::Function,
                    "method" => ProjectNodeKind::Method,
                    "type_alias" => ProjectNodeKind::TypeAlias,
                    "constant" => ProjectNodeKind::Constant,
                    "static" => ProjectNodeKind::Static,
                    "macro" => ProjectNodeKind::Macro,
                    "field" => ProjectNodeKind::Field,
                    _ => ProjectNodeKind::Other,
                };

                let span = start_line.map(|s| nodera_parser_core::SourceSpan {
                    start_line: s,
                    start_col: 1,
                    end_line: end_line.unwrap_or(s),
                    end_col: 1,
                    byte_offset: 0,
                    byte_len: 0,
                });

                let label = match kind {
                    ProjectNodeKind::Function => format!("fn {name}()"),
                    ProjectNodeKind::Struct => format!("struct {name}"),
                    ProjectNodeKind::Enum => format!("enum {name}"),
                    ProjectNodeKind::Trait => format!("trait {name}"),
                    ProjectNodeKind::Implementation => format!("impl {name}"),
                    _ => name.clone(),
                };

                Ok(ProjectNode {
                    id,
                    kind,
                    name,
                    label,
                    file,
                    visibility,
                    span,
                    signature,
                    doc,
                    parent_id,
                })
            })
            .map_err(|e| ProjectError::Database {
                reason: format!("Failed to execute search query: {e}"),
            })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r.map_err(|e| ProjectError::Database {
                reason: format!("Failed to read symbol row: {e}"),
            })?);
        }

        Ok(results)
    }
}
