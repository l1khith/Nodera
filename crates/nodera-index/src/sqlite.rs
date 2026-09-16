use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

use nodera_core::{IndexError, NoderaError, Result};
use nodera_markdown::{ParsedTask, Wikilink};

use crate::models::{IndexedTask, TagCount, TaskFilter};

pub const SCHEMA_VERSION: i64 = 1;

fn db_err(e: impl std::fmt::Display) -> NoderaError {
    IndexError::Database {
        reason: e.to_string(),
    }
    .into()
}

pub struct SqliteIndex {
    conn: Connection,
}

impl std::fmt::Debug for SqliteIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteIndex").finish()
    }
}

impl SqliteIndex {
    /// Opens or creates SQLite index at the given path, initializing schema.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(p).map_err(db_err)?;
        let index = Self { conn };
        index.init_schema()?;
        Ok(index)
    }

    /// Creates an in-memory SQLite database (ideal for tests).
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(db_err)?;
        let index = Self { conn };
        index.init_schema()?;
        Ok(index)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS files (
                id            TEXT PRIMARY KEY,
                path          TEXT NOT NULL UNIQUE,
                title         TEXT NOT NULL,
                extension     TEXT NOT NULL,
                content_hash  TEXT NOT NULL,
                size_bytes    INTEGER NOT NULL,
                modified_ns   INTEGER NOT NULL,
                indexed_at_ns INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS links (
                source_id     TEXT NOT NULL,
                target_path   TEXT NOT NULL,
                target_id     TEXT,
                start_offset  INTEGER,
                end_offset    INTEGER,
                PRIMARY KEY (source_id, target_path, start_offset)
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id            TEXT PRIMARY KEY,
                note_id       TEXT NOT NULL,
                line_number   INTEGER NOT NULL,
                checked       INTEGER NOT NULL,
                text          TEXT NOT NULL,
                due_date      TEXT
            );

            CREATE TABLE IF NOT EXISTS tags (
                note_id       TEXT NOT NULL,
                tag           TEXT NOT NULL,
                PRIMARY KEY (note_id, tag)
            );

            CREATE TABLE IF NOT EXISTS properties (
                note_id       TEXT NOT NULL,
                key           TEXT NOT NULL,
                value_json    TEXT NOT NULL,
                PRIMARY KEY (note_id, key)
            );

            CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
            CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_path);
            CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_note ON tasks(note_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_checked ON tasks(checked);
            CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);
            CREATE INDEX IF NOT EXISTS idx_tags_note ON tags(note_id);
            ",
            )
            .map_err(db_err)?;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?)",
                params![SCHEMA_VERSION.to_string()],
            )
            .map_err(db_err)?;

        Ok(())
    }

    /// Indexes or updates a note file's metadata and derived entities atomically.
    #[allow(clippy::too_many_arguments)]
    pub fn index_note_metadata(
        &mut self,
        note_id: &str,
        path: &str,
        title: &str,
        content_hash: &str,
        size_bytes: u64,
        modified_ns: u64,
        links: &[Wikilink],
        tasks: &[ParsedTask],
        tags: &[String],
        properties: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);

        let tx = self.conn.transaction().map_err(db_err)?;

        // 1. Files table
        tx.execute(
            "INSERT OR REPLACE INTO files (id, path, title, extension, content_hash, size_bytes, modified_ns, indexed_at_ns)
             VALUES (?, ?, ?, 'md', ?, ?, ?, ?)",
            params![
                note_id,
                path,
                title,
                content_hash,
                size_bytes as i64,
                modified_ns as i64,
                now_ns
            ],
        ).map_err(db_err)?;

        // 2. Clear existing child records for this note
        tx.execute("DELETE FROM links WHERE source_id = ?", params![note_id])
            .map_err(db_err)?;
        tx.execute("DELETE FROM tasks WHERE note_id = ?", params![note_id])
            .map_err(db_err)?;
        tx.execute("DELETE FROM tags WHERE note_id = ?", params![note_id])
            .map_err(db_err)?;
        tx.execute("DELETE FROM properties WHERE note_id = ?", params![note_id])
            .map_err(db_err)?;

        // 3. Links
        for link in links {
            tx.execute(
                "INSERT OR IGNORE INTO links (source_id, target_path, target_id, start_offset, end_offset)
                 VALUES (?, ?, NULL, ?, ?)",
                params![
                    note_id,
                    link.target,
                    link.start as i64,
                    link.end as i64,
                ],
            ).map_err(db_err)?;
        }

        // 4. Tasks
        for task in tasks {
            let task_id = format!("{note_id}:{}", task.line_number);
            tx.execute(
                "INSERT INTO tasks (id, note_id, line_number, checked, text, due_date)
                 VALUES (?, ?, ?, ?, ?, NULL)",
                params![
                    task_id,
                    note_id,
                    task.line_number as i64,
                    if task.checked { 1 } else { 0 },
                    task.text,
                ],
            )
            .map_err(db_err)?;
        }

        // 5. Tags
        for tag in tags {
            tx.execute(
                "INSERT OR IGNORE INTO tags (note_id, tag) VALUES (?, ?)",
                params![note_id, tag],
            )
            .map_err(db_err)?;
        }

        // 6. Properties
        for (key, val) in properties {
            let json_str = serde_json::to_string(val).unwrap_or_default();
            tx.execute(
                "INSERT OR REPLACE INTO properties (note_id, key, value_json) VALUES (?, ?, ?)",
                params![note_id, key, json_str],
            )
            .map_err(db_err)?;
        }

        tx.commit().map_err(db_err)?;
        debug!(note_id, path, "Indexed note metadata in SQLite");
        Ok(())
    }

    /// Removes all records associated with a deleted file.
    pub fn remove_file(&mut self, path: &str) -> Result<()> {
        let note_id: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM files WHERE path = ?",
                params![path],
                |row| row.get(0),
            )
            .optional()
            .map_err(db_err)?;

        if let Some(id) = note_id {
            let tx = self.conn.transaction().map_err(db_err)?;
            tx.execute("DELETE FROM links WHERE source_id = ?", params![id])
                .map_err(db_err)?;
            tx.execute("DELETE FROM tasks WHERE note_id = ?", params![id])
                .map_err(db_err)?;
            tx.execute("DELETE FROM tags WHERE note_id = ?", params![id])
                .map_err(db_err)?;
            tx.execute("DELETE FROM properties WHERE note_id = ?", params![id])
                .map_err(db_err)?;
            tx.execute("DELETE FROM files WHERE id = ?", params![id])
                .map_err(db_err)?;
            tx.commit().map_err(db_err)?;
            info!(path, "Removed file and derived entities from SQLite");
        }

        Ok(())
    }

    /// Queries tasks with filtering criteria across the vault.
    pub fn query_tasks(&self, filter: &TaskFilter) -> Result<Vec<IndexedTask>> {
        let mut query = String::from(
            "SELECT t.id, t.note_id, f.path, f.title, t.line_number, t.checked, t.text, t.due_date
             FROM tasks t
             JOIN files f ON t.note_id = f.id
             WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(checked) = filter.checked {
            query.push_str(" AND t.checked = ?");
            params_vec.push(Box::new(if checked { 1 } else { 0 }));
        }

        if let Some(note_path) = &filter.note_path {
            query.push_str(" AND f.path = ?");
            params_vec.push(Box::new(note_path.clone()));
        }

        if let Some(q) = &filter.search_query {
            query.push_str(" AND t.text LIKE ?");
            params_vec.push(Box::new(format!("%{q}%")));
        }

        query.push_str(" ORDER BY f.path ASC, t.line_number ASC");

        let mut stmt = self.conn.prepare(&query).map_err(db_err)?;
        let rusqlite_params: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();

        let rows = stmt
            .query_map(rusqlite_params.as_slice(), |row| {
                let checked_int: i32 = row.get(5)?;
                Ok(IndexedTask {
                    id: row.get(0)?,
                    note_id: row.get(1)?,
                    note_path: row.get(2)?,
                    note_title: row.get(3)?,
                    line_number: row.get::<_, i64>(4)? as usize,
                    checked: checked_int != 0,
                    text: row.get(6)?,
                    due_date: row.get(7)?,
                })
            })
            .map_err(db_err)?;

        let mut tasks = Vec::new();
        for task_res in rows {
            tasks.push(task_res.map_err(db_err)?);
        }
        Ok(tasks)
    }

    /// Queries all distinct tags and count of notes using them.
    pub fn query_tags(&self) -> Result<Vec<TagCount>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT tag, COUNT(note_id) as cnt
             FROM tags
             GROUP BY tag
             ORDER BY cnt DESC, tag ASC",
            )
            .map_err(db_err)?;

        let rows = stmt
            .query_map([], |row| {
                Ok(TagCount {
                    tag: row.get(0)?,
                    count: row.get::<_, i64>(1)? as usize,
                })
            })
            .map_err(db_err)?;

        let mut tags = Vec::new();
        for t in rows {
            tags.push(t.map_err(db_err)?);
        }
        Ok(tags)
    }

    /// Queries all note paths and their associated tags.
    pub fn query_all_note_tags(
        &self,
    ) -> Result<std::collections::HashMap<std::path::PathBuf, Vec<String>>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT f.path, t.tag
                 FROM tags t
                 JOIN files f ON t.note_id = f.id
                 ORDER BY f.path ASC, t.tag ASC",
            )
            .map_err(db_err)?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(db_err)?;

        let mut map: std::collections::HashMap<std::path::PathBuf, Vec<String>> =
            std::collections::HashMap::new();
        for r in rows {
            let (path, tag) = r.map_err(db_err)?;
            map.entry(std::path::PathBuf::from(path))
                .or_default()
                .push(tag);
        }
        Ok(map)
    }

    /// Queries paths of notes that contain links to target_name or target_path.
    pub fn query_backlinks(&self, target_name: &str) -> Result<Vec<String>> {
        let clean_target = target_name.trim_end_matches(".md");
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT f.path
             FROM links l
             JOIN files f ON l.source_id = f.id
             WHERE l.target_path = ?
                OR l.target_path = ?
                OR l.target_path LIKE ?
             ORDER BY f.path ASC",
            )
            .map_err(db_err)?;

        let with_md = format!("{clean_target}.md");
        let like_pattern = format!("%/{clean_target}");

        let rows = stmt
            .query_map(params![clean_target, with_md, like_pattern], |row| {
                row.get::<_, String>(0)
            })
            .map_err(db_err)?;

        let mut paths = Vec::new();
        for p in rows {
            paths.push(p.map_err(db_err)?);
        }
        Ok(paths)
    }

    /// Clears all tables for full rebuild.
    pub fn clear_all(&mut self) -> Result<()> {
        self.conn
            .execute_batch(
                "
            DELETE FROM links;
            DELETE FROM tasks;
            DELETE FROM tags;
            DELETE FROM properties;
            DELETE FROM files;
            ",
            )
            .map_err(db_err)?;
        info!("Cleared all SQLite index tables");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_index_operations() {
        let mut idx = SqliteIndex::in_memory().unwrap();

        let tasks = vec![
            ParsedTask {
                line_number: 5,
                checked: false,
                text: "Build search".to_string(),
                raw_line: "- [ ] Build search".to_string(),
            },
            ParsedTask {
                line_number: 10,
                checked: true,
                text: "Design schema".to_string(),
                raw_line: "- [x] Design schema".to_string(),
            },
        ];

        let links = vec![Wikilink {
            raw: "[[Architecture]]".to_string(),
            target: "Architecture".to_string(),
            display_text: None,
            start: 20,
            end: 36,
        }];

        let tags = vec!["rust".to_string(), "nodera".to_string()];
        let mut props = HashMap::new();
        props.insert("author".to_string(), serde_json::json!("Ailik"));

        idx.index_note_metadata(
            "note-1",
            "Notes/Plan.md",
            "Plan",
            "hash123",
            1024,
            100000,
            &links,
            &tasks,
            &tags,
            &props,
        )
        .unwrap();

        // 1. Query tasks
        let all_tasks = idx.query_tasks(&TaskFilter::default()).unwrap();
        assert_eq!(all_tasks.len(), 2);

        let incomplete = idx
            .query_tasks(&TaskFilter {
                checked: Some(false),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(incomplete.len(), 1);
        assert_eq!(incomplete[0].text, "Build search");

        // 2. Query tags
        let tag_counts = idx.query_tags().unwrap();
        assert_eq!(tag_counts.len(), 2);
        assert!(tag_counts.iter().any(|t| t.tag == "rust" && t.count == 1));

        // 3. Query backlinks
        let backlinks = idx.query_backlinks("Architecture").unwrap();
        assert_eq!(backlinks, vec!["Notes/Plan.md".to_string()]);

        // 4. Remove file
        idx.remove_file("Notes/Plan.md").unwrap();
        assert!(idx.query_tasks(&TaskFilter::default()).unwrap().is_empty());
        assert!(idx.query_tags().unwrap().is_empty());
    }
}
