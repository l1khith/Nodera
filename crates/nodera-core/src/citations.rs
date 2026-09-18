use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a single bibliographic entry from a BibTeX source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibEntry {
    /// Unique citation key used in Markdown (e.g. `@vaswani2017attention`)
    pub citation_key: String,
    /// BibTeX type: "article", "book", "inproceedings", "misc", "phdthesis", etc.
    pub entry_type: String,
    /// Primary title of the publication
    pub title: Option<String>,
    /// Parsed list of author names
    pub authors: Vec<String>,
    /// Publication year (e.g. "2023")
    pub year: Option<String>,
    /// Journal name or book title
    pub journal_or_book: Option<String>,
    /// Volume number
    pub volume: Option<String>,
    /// Page numbers or range (e.g. "12--25")
    pub pages: Option<String>,
    /// Digital Object Identifier (DOI)
    pub doi: Option<String>,
    /// URL link to paper or repository
    pub url: Option<String>,
    /// Abstract or synopsis
    pub abstract_text: Option<String>,
    /// File path where this entry originated
    pub source_file: Option<PathBuf>,
    /// All raw key-value fields preserving unmodeled attributes
    pub raw_fields: HashMap<String, String>,
}

impl BibEntry {
    /// Generates a human-friendly inline citation string, e.g. `(Vaswani et al., 2017)`.
    pub fn display_citation(&self) -> String {
        let author_str = if self.authors.is_empty() {
            self.title.as_deref().unwrap_or(&self.citation_key)
        } else if self.authors.len() == 1 {
            &self.authors[0]
        } else if self.authors.len() == 2 {
            return format!(
                "({} & {}, {})",
                self.authors[0],
                self.authors[1],
                self.year.as_deref().unwrap_or("n.d.")
            );
        } else {
            return format!(
                "({} et al., {})",
                self.authors[0],
                self.year.as_deref().unwrap_or("n.d.")
            );
        };

        format!(
            "({}, {})",
            author_str,
            self.year.as_deref().unwrap_or("n.d.")
        )
    }

    /// Formats a complete APA/Chicago-style reference description.
    pub fn formatted_reference(&self) -> String {
        let mut out = String::new();

        // Authors
        if !self.authors.is_empty() {
            out.push_str(&self.authors.join(", "));
            out.push(' ');
        }

        // Year
        if let Some(ref y) = self.year {
            out.push_str(&format!("({}). ", y));
        } else {
            out.push_str("(n.d.). ");
        }

        // Title
        if let Some(ref t) = self.title {
            out.push_str(t);
            if !t.ends_with('.') && !t.ends_with('?') && !t.ends_with('!') {
                out.push('.');
            }
            out.push(' ');
        }

        // Journal or Book
        if let Some(ref j) = self.journal_or_book {
            out.push_str(&format!("*{}*", j));
            if let Some(ref v) = self.volume {
                out.push_str(&format!(", {}", v));
            }
            if let Some(ref p) = self.pages {
                out.push_str(&format!(", {}", p));
            }
            out.push('.');
        }

        // DOI or URL
        if let Some(ref doi) = self.doi {
            out.push_str(&format!(" https://doi.org/{}", doi));
        } else if let Some(ref url) = self.url {
            out.push_str(&format!(" {}", url));
        }

        out
    }
}

/// Collection of bibliographic references indexed across a vault.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BibLibrary {
    pub entries: Vec<BibEntry>,
    pub source_files: Vec<PathBuf>,
}

impl BibLibrary {
    /// Creates an empty library.
    pub fn new() -> Self {
        Self::default()
    }

    /// Scans a vault directory for all `.bib` files and consolidates their entries.
    pub fn from_vault(vault_root: &Path) -> Self {
        let mut lib = Self::new();
        let mut bib_files = Vec::new();

        find_bib_files(vault_root, &mut bib_files);

        for path in bib_files {
            if let Ok(content) = fs::read_to_string(&path) {
                let parsed = parse_bibtex_str(&content, Some(&path));
                for entry in parsed {
                    if !lib
                        .entries
                        .iter()
                        .any(|e| e.citation_key == entry.citation_key)
                    {
                        lib.entries.push(entry);
                    }
                }
                lib.source_files.push(path);
            }
        }

        lib.entries
            .sort_by(|a, b| a.citation_key.cmp(&b.citation_key));
        lib
    }

    /// Finds an entry matching the given citation key.
    pub fn find_by_key(&self, key: &str) -> Option<&BibEntry> {
        let clean_key = key.trim_start_matches('@');
        self.entries
            .iter()
            .find(|e| e.citation_key.eq_ignore_ascii_case(clean_key))
    }

    /// Filters entries by a query string matching key, title, authors, or year.
    pub fn search(&self, query: &str) -> Vec<&BibEntry> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return self.entries.iter().collect();
        }

        self.entries
            .iter()
            .filter(|e| {
                e.citation_key.to_lowercase().contains(&q)
                    || e.title
                        .as_deref()
                        .map(|t| t.to_lowercase().contains(&q))
                        .unwrap_or(false)
                    || e.authors.iter().any(|a| a.to_lowercase().contains(&q))
                    || e.year.as_deref().map(|y| y.contains(&q)).unwrap_or(false)
                    || e.journal_or_book
                        .as_deref()
                        .map(|j| j.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .collect()
    }
}

/// Recursively discovers `.bib` files, skipping hidden or system directories.
fn find_bib_files(dir: &Path, acc: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }

        if path.is_dir() {
            find_bib_files(&path, acc);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("bib") {
            acc.push(path);
        }
    }
}

/// Parses a BibTeX string into structured `BibEntry` instances.
pub fn parse_bibtex_str(input: &str, source_file: Option<&Path>) -> Vec<BibEntry> {
    let mut entries = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some(&(_, ch)) = chars.peek() {
        if ch == '@' {
            chars.next(); // consume '@'

            // Read entry type
            let mut entry_type = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    entry_type.push(c);
                    chars.next();
                } else {
                    break;
                }
            }

            // Skip whitespace up to opening delimiter '{' or '('
            skip_whitespace(&mut chars);
            let delim_close = match chars.peek() {
                Some(&(_, '{')) => {
                    chars.next();
                    '}'
                }
                Some(&(_, '(')) => {
                    chars.next();
                    ')'
                }
                _ => continue,
            };

            let entry_type_lower = entry_type.to_lowercase();
            // Skip @comment or @string or @preamble for now
            if entry_type_lower == "comment"
                || entry_type_lower == "string"
                || entry_type_lower == "preamble"
            {
                skip_until_matching_close(&mut chars, delim_close);
                continue;
            }

            // Read citation key
            skip_whitespace(&mut chars);
            let mut citation_key = String::new();
            while let Some(&(_, c)) = chars.peek() {
                if c == ',' || c == delim_close || c.is_whitespace() {
                    break;
                }
                citation_key.push(c);
                chars.next();
            }

            skip_whitespace(&mut chars);
            if let Some(&(_, ',')) = chars.peek() {
                chars.next(); // consume comma after key
            }

            let mut raw_fields = HashMap::new();

            // Parse key = value fields until delim_close
            while chars.peek().is_some() {
                skip_whitespace(&mut chars);
                if let Some(&(_, next_c)) = chars.peek() {
                    if next_c == delim_close {
                        chars.next();
                        break;
                    }
                } else {
                    break;
                }

                // Field name
                let mut field_name = String::new();
                while let Some(&(_, fc)) = chars.peek() {
                    if fc.is_alphanumeric() || fc == '_' || fc == '-' {
                        field_name.push(fc);
                        chars.next();
                    } else {
                        break;
                    }
                }

                skip_whitespace(&mut chars);
                if let Some(&(_, '=')) = chars.peek() {
                    chars.next(); // consume '='
                } else {
                    // unexpected token, skip to next field or close
                    skip_to_next_field_or_close(&mut chars, delim_close);
                    continue;
                }

                skip_whitespace(&mut chars);
                let field_value = parse_bib_value(&mut chars, delim_close);
                if !field_name.is_empty() {
                    raw_fields.insert(field_name.to_lowercase(), field_value);
                }

                skip_whitespace(&mut chars);
                if let Some(&(_, ',')) = chars.peek() {
                    chars.next(); // consume comma
                }
            }

            if !citation_key.is_empty() {
                let title = raw_fields.get("title").map(|s| clean_latex(s));
                let authors = raw_fields
                    .get("author")
                    .map(|s| parse_authors(s))
                    .unwrap_or_default();
                let year = raw_fields.get("year").cloned().map(|s| clean_latex(&s));
                let journal_or_book = raw_fields
                    .get("journal")
                    .or_else(|| raw_fields.get("booktitle"))
                    .map(|s| clean_latex(s));
                let volume = raw_fields.get("volume").cloned();
                let pages = raw_fields.get("pages").cloned();
                let doi = raw_fields
                    .get("doi")
                    .cloned()
                    .map(|d| d.trim_start_matches("https://doi.org/").to_string());
                let url = raw_fields.get("url").cloned();
                let abstract_text = raw_fields.get("abstract").map(|s| clean_latex(s));

                entries.push(BibEntry {
                    citation_key,
                    entry_type: entry_type_lower,
                    title,
                    authors,
                    year,
                    journal_or_book,
                    volume,
                    pages,
                    doi,
                    url,
                    abstract_text,
                    source_file: source_file.map(|p| p.to_path_buf()),
                    raw_fields,
                });
            }
        } else {
            chars.next();
        }
    }

    entries
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::CharIndices>) {
    while let Some(&(_, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn skip_until_matching_close(
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
    close_char: char,
) {
    let mut depth = 1;
    let open_char = if close_char == '}' { '{' } else { '(' };

    while let Some(&(_, c)) = chars.peek() {
        chars.next();
        if c == open_char {
            depth += 1;
        } else if c == close_char {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
    }
}

fn skip_to_next_field_or_close(
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
    close_char: char,
) {
    while let Some(&(_, c)) = chars.peek() {
        if c == ',' {
            chars.next();
            break;
        } else if c == close_char {
            break;
        }
        chars.next();
    }
}

fn parse_bib_value(
    chars: &mut std::iter::Peekable<std::str::CharIndices>,
    entry_close_delim: char,
) -> String {
    let mut val = String::new();

    match chars.peek() {
        Some(&(_, '{')) => {
            chars.next(); // consume opening '{'
            let mut depth = 1;
            while let Some(&(_, c)) = chars.peek() {
                chars.next();
                if c == '{' {
                    depth += 1;
                    val.push(c);
                } else if c == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    val.push(c);
                } else {
                    val.push(c);
                }
            }
        }
        Some(&(_, '"')) => {
            chars.next(); // consume opening '"'
            let mut escaped = false;
            while let Some(&(_, c)) = chars.peek() {
                chars.next();
                if escaped {
                    val.push(c);
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    break;
                } else {
                    val.push(c);
                }
            }
        }
        _ => {
            // Raw number or token up to comma, whitespace, or close delimiter
            while let Some(&(_, c)) = chars.peek() {
                if c == ',' || c == entry_close_delim || c == '\n' || c == '\r' {
                    break;
                }
                val.push(c);
                chars.next();
            }
        }
    }

    val.trim().to_string()
}

/// Splits "Author One and Author Two and Last, First" into clean names.
pub fn parse_authors(raw: &str) -> Vec<String> {
    raw.split(" and ")
        .map(|a| a.trim())
        .filter(|a| !a.is_empty())
        .map(|author| {
            let cleaned = clean_latex(author);
            if cleaned.contains(',') {
                let parts: Vec<&str> = cleaned.splitn(2, ',').collect();
                if parts.len() == 2 {
                    format!("{} {}", parts[1].trim(), parts[0].trim())
                } else {
                    cleaned
                }
            } else {
                cleaned
            }
        })
        .collect()
}

/// Cleans common LaTeX accents, delimiters, and typography escapes.
pub fn clean_latex(input: &str) -> String {
    let mut s = input.to_string();

    // Remove outermost braces if wrapping entire string
    if s.starts_with('{') && s.ends_with('}') && s.len() >= 2 {
        s = s[1..s.len() - 1].to_string();
    }

    let replacements = [
        ("{\\\"o}", "ö"),
        ("{\\\"a}", "ä"),
        ("{\\\"u}", "ü"),
        ("{\\\"O}", "Ö"),
        ("{\\\"A}", "Ä"),
        ("{\\\"U}", "Ü"),
        ("{\\'e}", "é"),
        ("{\\`e}", "è"),
        ("{\\'a}", "á"),
        ("{\\`a}", "à"),
        ("{\\^a}", "â"),
        ("{\\~n}", "ñ"),
        ("{\\ss}", "ß"),
        ("{\\&}", "&"),
        ("\\&", "&"),
        ("\\%", "%"),
        ("\\$", "$"),
        ("--", "–"),
        ("---", "—"),
        ("~", " "),
    ];

    for (from, to) in replacements {
        s = s.replace(from, to);
    }

    // Strip inline capitalization preservation braces e.g. {BERT} -> BERT
    s = s.replace(['{', '}'], "");

    // Normalize multiple spaces / newlines
    let words: Vec<&str> = s.split_whitespace().collect();
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_BIB: &str = r#"
@article{vaswani2017attention,
  author    = {Vaswani, Ashish and Shazeer, Noam and Parmar, Niki},
  title     = {{Attention Is All You Need}},
  journal   = {Advances in Neural Information Processing Systems},
  year      = {2017},
  volume    = {30},
  pages     = {5998--6008},
  doi       = {10.5555/3295222.3295349}
}

@book{rustbook,
  author    = {Steve Klabnik and Carol Nichols},
  title     = {The Rust Programming Language},
  year      = 2018,
  publisher = {No Starch Press}
}

@misc{singleauthor,
  author    = {Turing, Alan M.},
  title     = {Computing Machinery and Intelligence},
  year      = {1950}
}
"#;

    #[test]
    fn test_parse_bibtex_str() {
        let entries = parse_bibtex_str(SAMPLE_BIB, None);
        assert_eq!(entries.len(), 3);

        // Entry 1
        let e1 = &entries[0];
        assert_eq!(e1.citation_key, "vaswani2017attention");
        assert_eq!(e1.entry_type, "article");
        assert_eq!(e1.title.as_deref(), Some("Attention Is All You Need"));
        assert_eq!(e1.authors.len(), 3);
        assert_eq!(e1.authors[0], "Ashish Vaswani");
        assert_eq!(e1.year.as_deref(), Some("2017"));
        assert_eq!(
            e1.journal_or_book.as_deref(),
            Some("Advances in Neural Information Processing Systems")
        );
        assert_eq!(e1.doi.as_deref(), Some("10.5555/3295222.3295349"));
        assert_eq!(e1.display_citation(), "(Ashish Vaswani et al., 2017)");

        // Entry 2 (two authors, raw integer year)
        let e2 = &entries[1];
        assert_eq!(e2.citation_key, "rustbook");
        assert_eq!(e2.entry_type, "book");
        assert_eq!(e2.authors.len(), 2);
        assert_eq!(
            e2.display_citation(),
            "(Steve Klabnik & Carol Nichols, 2018)"
        );

        // Entry 3 (single author, comma format)
        let e3 = &entries[2];
        assert_eq!(e3.citation_key, "singleauthor");
        assert_eq!(e3.authors, vec!["Alan M. Turing"]);
        assert_eq!(e3.display_citation(), "(Alan M. Turing, 1950)");
    }

    #[test]
    fn test_clean_latex() {
        assert_eq!(clean_latex("Sch{\\\"o}n and Caf{\\'e}"), "Schön and Café");
        assert_eq!(
            clean_latex("{BERT}: Pre-training of Deep Bidirectional Transformers"),
            "BERT: Pre-training of Deep Bidirectional Transformers"
        );
        assert_eq!(clean_latex("Rock~\\&~Roll"), "Rock & Roll");
    }

    #[test]
    fn test_bib_library_search() {
        let entries = parse_bibtex_str(SAMPLE_BIB, None);
        let mut lib = BibLibrary::new();
        lib.entries = entries;

        let res1 = lib.search("attention");
        assert_eq!(res1.len(), 1);
        assert_eq!(res1[0].citation_key, "vaswani2017attention");

        let res2 = lib.search("klabnik");
        assert_eq!(res2.len(), 1);
        assert_eq!(res2[0].citation_key, "rustbook");

        let res3 = lib.search("1950");
        assert_eq!(res3.len(), 1);
        assert_eq!(res3[0].citation_key, "singleauthor");

        let res_empty = lib.search("");
        assert_eq!(res_empty.len(), 3);
    }
}
