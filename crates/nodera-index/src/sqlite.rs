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

/// Borrowed note metadata record for batch indexing operations in SQLite.
#[derive(Debug, Clone)]
pub struct NoteMetadataRecord<'a> {
    pub note_id: &'a str,
    pub path: &'a str,
    pub title: &'a str,
    pub content_hash: &'a str,
    pub size_bytes: u64,
    pub modified_ns: u64,
    pub links: &'a [Wikilink],
    pub tasks: &'a [ParsedTask],
    pub tags: &'a [String],
    pub properties: &'a HashMap<String, serde_json::Value>,
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

    /// Atomically clears and rebuilds the SQLite index with the given records inside a single transaction.
    /// Uses prepared statements for peak throughput.
    /// If any insert fails, the transaction is cleanly rolled back with zero partial state.
    pub fn rebuild_batch(&mut self, records: &[NoteMetadataRecord<'_>]) -> Result<()> {
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);

        let tx = self.conn.transaction().map_err(db_err)?;

        // 1. Clear all existing records inside the transaction
        tx.execute_batch(
            "
            DELETE FROM links;
            DELETE FROM tasks;
            DELETE FROM tags;
            DELETE FROM properties;
            DELETE FROM files;
            ",
        )
        .map_err(db_err)?;

        // 2. Prepare statements once for all records
        {
            let mut stmt_file = tx
                .prepare_cached(
                    "INSERT OR REPLACE INTO files (id, path, title, extension, content_hash, size_bytes, modified_ns, indexed_at_ns)
                     VALUES (?, ?, ?, 'md', ?, ?, ?, ?)",
                )
                .map_err(db_err)?;

            let mut stmt_link = tx
                .prepare_cached(
                    "INSERT OR IGNORE INTO links (source_id, target_path, target_id, start_offset, end_offset)
                     VALUES (?, ?, NULL, ?, ?)",
                )
                .map_err(db_err)?;

            let mut stmt_task = tx
                .prepare_cached(
                    "INSERT INTO tasks (id, note_id, line_number, checked, text, due_date)
                     VALUES (?, ?, ?, ?, ?, NULL)",
                )
                .map_err(db_err)?;

            let mut stmt_tag = tx
                .prepare_cached("INSERT OR IGNORE INTO tags (note_id, tag) VALUES (?, ?)")
                .map_err(db_err)?;

            let mut stmt_prop = tx
                .prepare_cached(
                    "INSERT OR REPLACE INTO properties (note_id, key, value_json) VALUES (?, ?, ?)",
                )
                .map_err(db_err)?;

            for record in records {
                stmt_file
                    .execute(params![
                        record.note_id,
                        record.path,
                        record.title,
                        record.content_hash,
                        record.size_bytes as i64,
                        record.modified_ns as i64,
                        now_ns,
                    ])
                    .map_err(db_err)?;

                for link in record.links {
                    stmt_link
                        .execute(params![
                            record.note_id,
                            link.target,
                            link.start as i64,
                            link.end as i64,
                        ])
                        .map_err(db_err)?;
                }

                for task in record.tasks {
                    let task_id = format!("{}:{}", record.note_id, task.line_number);
                    stmt_task
                        .execute(params![
                            task_id,
                            record.note_id,
                            task.line_number as i64,
                            if task.checked { 1 } else { 0 },
                            task.text,
                        ])
                        .map_err(db_err)?;
                }

                for tag in record.tags {
                    stmt_tag
                        .execute(params![record.note_id, tag])
                        .map_err(db_err)?;
                }

                for (key, val) in record.properties {
                    let json_str = serde_json::to_string(val).unwrap_or_default();
                    stmt_prop
                        .execute(params![record.note_id, key, json_str])
                        .map_err(db_err)?;
                }
            }
        }

        tx.commit().map_err(db_err)?;
        info!(
            indexed_notes = records.len(),
            "Committed batch rebuild in SQLite"
        );
        Ok(())
    }

    /// Incrementally batches an array of note records into SQLite inside a single transaction without clearing existing tables.
    pub fn index_notes_batch(&mut self, records: &[NoteMetadataRecord<'_>]) -> Result<()> {
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);

        let tx = self.conn.transaction().map_err(db_err)?;

        {
            let mut stmt_file = tx
                .prepare_cached(
                    "INSERT OR REPLACE INTO files (id, path, title, extension, content_hash, size_bytes, modified_ns, indexed_at_ns)
                     VALUES (?, ?, ?, 'md', ?, ?, ?, ?)",
                )
                .map_err(db_err)?;

            let mut stmt_del_links = tx
                .prepare_cached("DELETE FROM links WHERE source_id = ?")
                .map_err(db_err)?;
            let mut stmt_del_tasks = tx
                .prepare_cached("DELETE FROM tasks WHERE note_id = ?")
                .map_err(db_err)?;
            let mut stmt_del_tags = tx
                .prepare_cached("DELETE FROM tags WHERE note_id = ?")
                .map_err(db_err)?;
            let mut stmt_del_props = tx
                .prepare_cached("DELETE FROM properties WHERE note_id = ?")
                .map_err(db_err)?;

            let mut stmt_link = tx
                .prepare_cached(
                    "INSERT OR IGNORE INTO links (source_id, target_path, target_id, start_offset, end_offset)
                     VALUES (?, ?, NULL, ?, ?)",
                )
                .map_err(db_err)?;

            let mut stmt_task = tx
                .prepare_cached(
                    "INSERT INTO tasks (id, note_id, line_number, checked, text, due_date)
                     VALUES (?, ?, ?, ?, ?, NULL)",
                )
                .map_err(db_err)?;

            let mut stmt_tag = tx
                .prepare_cached("INSERT OR IGNORE INTO tags (note_id, tag) VALUES (?, ?)")
                .map_err(db_err)?;

            let mut stmt_prop = tx
                .prepare_cached(
                    "INSERT OR REPLACE INTO properties (note_id, key, value_json) VALUES (?, ?, ?)",
                )
                .map_err(db_err)?;

            for record in records {
                stmt_file
                    .execute(params![
                        record.note_id,
                        record.path,
                        record.title,
                        record.content_hash,
                        record.size_bytes as i64,
                        record.modified_ns as i64,
                        now_ns,
                    ])
                    .map_err(db_err)?;

                stmt_del_links
                    .execute(params![record.note_id])
                    .map_err(db_err)?;
                stmt_del_tasks
                    .execute(params![record.note_id])
                    .map_err(db_err)?;
                stmt_del_tags
                    .execute(params![record.note_id])
                    .map_err(db_err)?;
                stmt_del_props
                    .execute(params![record.note_id])
                    .map_err(db_err)?;

                for link in record.links {
                    stmt_link
                        .execute(params![
                            record.note_id,
                            link.target,
                            link.start as i64,
                            link.end as i64,
                        ])
                        .map_err(db_err)?;
                }

                for task in record.tasks {
                    let task_id = format!("{}:{}", record.note_id, task.line_number);
                    stmt_task
                        .execute(params![
                            task_id,
                            record.note_id,
                            task.line_number as i64,
                            if task.checked { 1 } else { 0 },
                            task.text,
                        ])
                        .map_err(db_err)?;
                }

                for tag in record.tags {
                    stmt_tag
                        .execute(params![record.note_id, tag])
                        .map_err(db_err)?;
                }

                for (key, val) in record.properties {
                    let json_str = serde_json::to_string(val).unwrap_or_default();
                    stmt_prop
                        .execute(params![record.note_id, key, json_str])
                        .map_err(db_err)?;
                }
            }
        }

        tx.commit().map_err(db_err)?;
        debug!(
            count = records.len(),
            "Committed incremental batch in SQLite"
        );
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

    /// Queries all tags associated with a specific note path.
    pub fn query_tags_for_path(&self, path: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.tag
                 FROM tags t
                 JOIN files f ON t.note_id = f.id
                 WHERE f.path = ?
                 ORDER BY t.tag ASC",
            )
            .map_err(db_err)?;

        let rows = stmt
            .query_map(params![path], |row| row.get::<_, String>(0))
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

    #[test]
    fn test_sqlite_batch_index_success() {
        let mut idx = SqliteIndex::in_memory().unwrap();
        let tasks1 = vec![ParsedTask {
            line_number: 1,
            checked: false,
            text: "Task A".to_string(),
            raw_line: "- [ ] Task A".to_string(),
        }];
        let tasks2 = vec![ParsedTask {
            line_number: 2,
            checked: true,
            text: "Task B".to_string(),
            raw_line: "- [x] Task B".to_string(),
        }];
        let links1 = vec![Wikilink {
            raw: "[[Note2]]".to_string(),
            target: "Note2".to_string(),
            display_text: None,
            start: 0,
            end: 9,
        }];
        let tags1 = vec!["tag1".to_string()];
        let tags2 = vec!["tag2".to_string()];
        let props = HashMap::new();

        let records = vec![
            NoteMetadataRecord {
                note_id: "id-1",
                path: "Note1.md",
                title: "Note 1",
                content_hash: "h1",
                size_bytes: 100,
                modified_ns: 1000,
                links: &links1,
                tasks: &tasks1,
                tags: &tags1,
                properties: &props,
            },
            NoteMetadataRecord {
                note_id: "id-2",
                path: "Note2.md",
                title: "Note 2",
                content_hash: "h2",
                size_bytes: 200,
                modified_ns: 2000,
                links: &[],
                tasks: &tasks2,
                tags: &tags2,
                properties: &props,
            },
        ];

        idx.rebuild_batch(&records).unwrap();

        let tasks = idx.query_tasks(&TaskFilter::default()).unwrap();
        assert_eq!(tasks.len(), 2);
        let tags = idx.query_tags().unwrap();
        assert_eq!(tags.len(), 2);
        let backlinks = idx.query_backlinks("Note2").unwrap();
        assert_eq!(backlinks, vec!["Note1.md".to_string()]);
    }

    #[test]
    fn test_sqlite_batch_rollback_on_failure_and_no_partial_state() {
        let mut idx = SqliteIndex::in_memory().unwrap();
        let tasks1 = vec![ParsedTask {
            line_number: 1,
            checked: false,
            text: "Initial task".to_string(),
            raw_line: "- [ ] Initial task".to_string(),
        }];
        let tags1 = vec!["initial".to_string()];
        let props = HashMap::new();

        // Populate with 1 good note first
        let initial_records = vec![NoteMetadataRecord {
            note_id: "init-1",
            path: "Initial.md",
            title: "Initial",
            content_hash: "h0",
            size_bytes: 50,
            modified_ns: 500,
            links: &[],
            tasks: &tasks1,
            tags: &tags1,
            properties: &props,
        }];
        idx.rebuild_batch(&initial_records).unwrap();

        // Verify initial note exists
        assert_eq!(idx.query_tasks(&TaskFilter::default()).unwrap().len(), 1);
        assert_eq!(idx.query_tags().unwrap().len(), 1);

        // Prepare a batch that triggers an error (two tasks with the SAME line number in the same note -> duplicate primary key in tasks table)
        let duplicate_tasks = vec![
            ParsedTask {
                line_number: 42,
                checked: false,
                text: "Task 1".to_string(),
                raw_line: "- [ ] Task 1".to_string(),
            },
            ParsedTask {
                line_number: 42, // Duplicate task_id will fail PRIMARY KEY (id) constraint!
                checked: true,
                text: "Task 2 (duplicate id)".to_string(),
                raw_line: "- [x] Task 2 (duplicate id)".to_string(),
            },
        ];
        let bad_tags = vec!["bad".to_string()];

        let bad_records = vec![NoteMetadataRecord {
            note_id: "bad-1",
            path: "Bad.md",
            title: "Bad Note",
            content_hash: "h_bad",
            size_bytes: 500,
            modified_ns: 9999,
            links: &[],
            tasks: &duplicate_tasks,
            tags: &bad_tags,
            properties: &props,
        }];

        // Execute rebuild_batch with the failing batch
        let res = idx.rebuild_batch(&bad_records);
        assert!(
            res.is_err(),
            "Batch rebuild must fail on constraint violation"
        );

        // Verify total rollback: initial state is NOT deleted and partial bad state is NOT inserted!
        let tasks_after = idx.query_tasks(&TaskFilter::default()).unwrap();
        assert_eq!(
            tasks_after.len(),
            1,
            "Initial tasks must be restored by rollback"
        );
        assert_eq!(tasks_after[0].text, "Initial task");

        let tags_after = idx.query_tags().unwrap();
        assert_eq!(tags_after.len(), 1, "Initial tags must be restored");
        assert_eq!(tags_after[0].tag, "initial");

        // Ensure NO partial records from bad note exist in any table
        assert!(idx.query_backlinks("Bad").unwrap().is_empty());
    }

    #[test]
    fn test_sqlite_batch_deterministic_repeat_rebuild() {
        let mut idx = SqliteIndex::in_memory().unwrap();
        let tasks = vec![ParsedTask {
            line_number: 5,
            checked: true,
            text: "Task".to_string(),
            raw_line: "- [x] Task".to_string(),
        }];
        let links = vec![Wikilink {
            raw: "[[Target]]".to_string(),
            target: "Target".to_string(),
            display_text: None,
            start: 0,
            end: 10,
        }];
        let tags = vec!["deterministic".to_string()];
        let mut props = HashMap::new();
        props.insert("key".to_string(), serde_json::json!("value"));

        let records = vec![NoteMetadataRecord {
            note_id: "note-1",
            path: "Note.md",
            title: "Title",
            content_hash: "hash_det",
            size_bytes: 123,
            modified_ns: 456,
            links: &links,
            tasks: &tasks,
            tags: &tags,
            properties: &props,
        }];

        // First rebuild
        idx.rebuild_batch(&records).unwrap();
        let tasks_1 = idx.query_tasks(&TaskFilter::default()).unwrap();
        let tags_1 = idx.query_tags().unwrap();

        // Second rebuild with identical dataset
        idx.rebuild_batch(&records).unwrap();
        let tasks_2 = idx.query_tasks(&TaskFilter::default()).unwrap();
        let tags_2 = idx.query_tags().unwrap();

        assert_eq!(
            tasks_1, tasks_2,
            "Repeat rebuild must produce identical tasks"
        );
        assert_eq!(tags_1, tags_2, "Repeat rebuild must produce identical tags");
    }
}
