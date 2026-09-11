# Markdown and Note Specification

## 1. Source format

Primary note format: CommonMark/GFM-compatible Markdown, with documented Nodera extensions.

## 2. Wikilinks

Supported syntax:

```text
[[Note Name]]
[[folder/Note Name]]
[[Note Name|display text]]
```

Initial implementation should resolve by normalized vault-relative path/title according to a deterministic rule.

## 3. Tasks

Supported:

```markdown
- [ ] Open task
- [x] Completed task
```

Task state is represented directly in Markdown.

Optional future support:
- due dates
- priorities
- recurrence

These should not be added to V1 without a clear textual syntax specification.

## 4. Frontmatter

Optional YAML block:

```yaml
---
type: project
status: active
tags:
  - rust
  - desktop
---
```

Frontmatter is metadata, not required for ordinary notes.

## 5. Tags

Initial syntax:

```text
#rust
#project/pdf
```

Tag rules must avoid interpreting headings such as:

```markdown
# Heading
```

as tags.

## 6. Headings

Standard Markdown headings are used.

## 7. Attachments

Attachments live inside the vault and are referenced by relative paths.

## 8. PDF-generated notes

A generated note should remain ordinary Markdown.

Suggested frontmatter:

```yaml
---
source_type: pdf
source_file: "Book.pdf"
imported_by: nodera
---
```

Do not make generated metadata mandatory if it harms portability.

## 9. Page markers

Optional PDF import output:

```markdown
<!-- nodera:page=42 -->
```

This is an implementation option and should be disabled by default unless users request traceability.

## 10. Compatibility rule

If Nodera adds a syntax extension, opening the file in a normal Markdown editor should remain useful even if the extension is ignored.
