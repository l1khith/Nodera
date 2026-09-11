# Task Model

## 1. Philosophy

Tasks are not a separate proprietary task database.

The source is Markdown:

```markdown
- [ ] Implement search
- [x] Create vault
```

The global task view is an index.

## 2. Task identity

A task should have a stable internal identity when possible, but V1 can use note ID + location/content reconciliation.

Avoid adding hidden IDs into every Markdown line until a concrete requirement exists.

## 3. Task state

Initial states:
- open
- completed

Future:
- cancelled
- scheduled
- recurring

## 4. Task navigation

Global task result:

```text
□ Implement PDF parser
   Books/PDF Converter.md:42
```

Click → open source note at the relevant location.

## 5. Toggle behavior

When the user toggles:

```text
- [ ] Task
```

to:

```text
- [x] Task
```

Nodera updates the source Markdown and then updates derived indexes.

## 6. Task grouping

Default grouping:
- Today
- Upcoming
- No date

Only date-aware grouping should be introduced after a documented date syntax exists.

## 7. Projects

A project can be represented as a Markdown note:

```yaml
---
type: project
status: active
---
```

Tasks remain inside normal notes and can link to the project note.
