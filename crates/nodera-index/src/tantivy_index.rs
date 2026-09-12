use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, Term};
use tracing::{debug, info};

use crate::models::SearchResult;
use nodera_core::{NoderaError, Result, SearchError};

fn search_err(e: impl std::fmt::Display) -> NoderaError {
    SearchError::Engine {
        reason: e.to_string(),
    }
    .into()
}

pub struct TantivyIndex {
    index: Index,
    reader: IndexReader,
    writer: IndexWriter,
    f_id: Field,
    f_path: Field,
    f_title: Field,
    f_headings: Field,
    f_body: Field,
    f_tags: Field,
}

impl std::fmt::Debug for TantivyIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TantivyIndex").finish()
    }
}

impl TantivyIndex {
    /// Opens or creates a Tantivy index in the given filesystem directory.
    pub fn open(dir_path: impl AsRef<Path>) -> Result<Self> {
        let p = dir_path.as_ref();
        std::fs::create_dir_all(p).map_err(search_err)?;

        let (schema, f_id, f_path, f_title, f_headings, f_body, f_tags) = Self::build_schema();

        let index =
            if Index::exists(&tantivy::directory::MmapDirectory::open(p).map_err(search_err)?)
                .unwrap_or(false)
            {
                Index::open_in_dir(p).map_err(search_err)?
            } else {
                Index::create_in_dir(p, schema).map_err(search_err)?
            };

        Self::init_with_index(index, f_id, f_path, f_title, f_headings, f_body, f_tags)
    }

    /// Creates an in-memory Tantivy index (for testing).
    pub fn in_memory() -> Result<Self> {
        let (schema, f_id, f_path, f_title, f_headings, f_body, f_tags) = Self::build_schema();
        let index = Index::create_in_ram(schema);
        Self::init_with_index(index, f_id, f_path, f_title, f_headings, f_body, f_tags)
    }

    fn build_schema() -> (Schema, Field, Field, Field, Field, Field, Field) {
        let mut schema_builder = Schema::builder();
        let f_id = schema_builder.add_text_field("id", STRING | STORED);
        let f_path = schema_builder.add_text_field("path", STRING | STORED);
        let f_title = schema_builder.add_text_field("title", TEXT | STORED);
        let f_headings = schema_builder.add_text_field("headings", TEXT);
        let f_body = schema_builder.add_text_field("body", TEXT | STORED);
        let f_tags = schema_builder.add_text_field("tags", TEXT | STORED);
        let schema = schema_builder.build();
        (schema, f_id, f_path, f_title, f_headings, f_body, f_tags)
    }

    fn init_with_index(
        index: Index,
        f_id: Field,
        f_path: Field,
        f_title: Field,
        f_headings: Field,
        f_body: Field,
        f_tags: Field,
    ) -> Result<Self> {
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(search_err)?;

        let writer = index.writer(50_000_000).map_err(search_err)?;

        Ok(Self {
            index,
            reader,
            writer,
            f_id,
            f_path,
            f_title,
            f_headings,
            f_body,
            f_tags,
        })
    }

    /// Indexes or replaces a document in the full-text search index.
    pub fn index_document(
        &mut self,
        id: &str,
        path: &str,
        title: &str,
        headings: &str,
        body: &str,
        tags: &str,
    ) -> Result<()> {
        // Delete any existing doc with this path
        self.writer
            .delete_term(Term::from_field_text(self.f_path, path));

        self.writer
            .add_document(doc!(
                self.f_id => id,
                self.f_path => path,
                self.f_title => title,
                self.f_headings => headings,
                self.f_body => body,
                self.f_tags => tags,
            ))
            .map_err(search_err)?;

        debug!(path, title, "Added document to Tantivy");
        Ok(())
    }

    /// Removes a document by its relative path.
    pub fn delete_document(&mut self, path: &str) -> Result<()> {
        self.writer
            .delete_term(Term::from_field_text(self.f_path, path));
        info!(path, "Deleted document from Tantivy");
        Ok(())
    }

    /// Commits pending indexing operations so they are visible to searches.
    pub fn commit(&mut self) -> Result<()> {
        self.writer.commit().map_err(search_err)?;
        self.reader.reload().map_err(search_err)?;
        Ok(())
    }

    /// Executes full-text search across titles, headings, body, and tags.
    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let clean_query = query_str.trim();
        if clean_query.is_empty() {
            return Ok(Vec::new());
        }

        let searcher = self.reader.searcher();
        let mut query_parser = QueryParser::for_index(
            &self.index,
            vec![self.f_title, self.f_headings, self.f_body, self.f_tags],
        );

        // Boost title and headings over body text
        query_parser.set_field_boost(self.f_title, 3.0);
        query_parser.set_field_boost(self.f_headings, 2.0);
        query_parser.set_field_boost(self.f_tags, 1.5);

        let query = query_parser
            .parse_query(clean_query)
            .or_else(|_| query_parser.parse_query(&format!("*{clean_query}*")))
            .unwrap_or_else(|_| Box::new(tantivy::query::AllQuery));

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(search_err)?;

        let mut snippet_gen =
            tantivy::snippet::SnippetGenerator::create(&searcher, &*query, self.f_body).ok();

        if let Some(ref mut gen) = snippet_gen {
            gen.set_max_num_chars(120);
        }

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address).map_err(search_err)?;

            let id = retrieved_doc
                .get_first(self.f_id)
                .and_then(|v: &OwnedValue| v.as_str())
                .unwrap_or_default()
                .to_string();

            let path = retrieved_doc
                .get_first(self.f_path)
                .and_then(|v: &OwnedValue| v.as_str())
                .unwrap_or_default()
                .to_string();

            let title = retrieved_doc
                .get_first(self.f_title)
                .and_then(|v: &OwnedValue| v.as_str())
                .unwrap_or_default()
                .to_string();

            let snippet = if let Some(ref gen) = snippet_gen {
                let snip = gen.snippet_from_doc(&retrieved_doc);
                snip.to_html()
            } else {
                retrieved_doc
                    .get_first(self.f_body)
                    .and_then(|v: &OwnedValue| v.as_str())
                    .map(|b: &str| {
                        if b.len() > 120 {
                            format!("{}...", &b[..120])
                        } else {
                            b.to_string()
                        }
                    })
                    .unwrap_or_default()
            };

            results.push(SearchResult {
                note_id: id,
                path,
                title,
                score,
                snippet,
            });
        }

        Ok(results)
    }

    /// Clears all documents in Tantivy index.
    pub fn clear_all(&mut self) -> Result<()> {
        self.writer.delete_all_documents().map_err(search_err)?;
        info!("Cleared all Tantivy index documents");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tantivy_index_and_search() {
        let mut idx = TantivyIndex::in_memory().unwrap();

        idx.index_document(
            "1",
            "Notes/Rust.md",
            "Rust Concurrency",
            "Threads Channels Async",
            "Rust provides fearless concurrency without data races.",
            "programming systems",
        )
        .unwrap();

        idx.index_document(
            "2",
            "Notes/Python.md",
            "Python Asyncio",
            "Event Loop Coroutines",
            "Python uses an event loop for asynchronous programming.",
            "programming scripting",
        )
        .unwrap();

        idx.commit().unwrap();

        // Search for concurrency
        let res = idx.search("concurrency", 10).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].title, "Rust Concurrency");
        assert!(res[0].snippet.to_lowercase().contains("concurrency"));

        // Search for programming (both notes match)
        let res2 = idx.search("programming", 10).unwrap();
        assert_eq!(res2.len(), 2);

        // Delete note
        idx.delete_document("Notes/Rust.md").unwrap();
        idx.commit().unwrap();

        let res3 = idx.search("concurrency", 10).unwrap();
        assert!(res3.is_empty());
    }
}
