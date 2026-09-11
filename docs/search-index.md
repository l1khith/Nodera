# Search and Knowledge Index

## 1. Search goals

Search should feel like a universal entry point into:
- notes
- headings
- tasks
- projects
- books/documents
- tags
- properties

## 2. Search pipeline

```text
Query
 ↓
Query parser
 ↓
Tantivy search
 ↓
Metadata lookup
 ↓
Result ranking
 ↓
UI result
```

## 3. Search fields

Suggested:
- title
- path
- body
- heading
- tags

## 4. Ranking

Initial ranking:
1. exact title match
2. title prefix/term match
3. heading match
4. body match
5. path/tag match

Tune only after real usage data.

## 5. Snippets

Results should show a small relevant snippet.

Do not load the entire note merely to display a result if an indexed/stored snippet can be produced safely.

## 6. Backlinks

Backlinks are derived from `links`:

```text
source → target
```

For current note `B`:

```sql
SELECT source_id
FROM links
WHERE target_id = B;
```

Actual query design should use indexes appropriate to the final schema.

## 7. Graph

The graph uses the same link relationships.

Nodes:
- notes
- optional project/document nodes

Edges:
- Wikilinks

Tags/properties should not automatically become graph edges unless explicitly enabled.

## 8. Index consistency

The source file wins.

If index content differs from the file:
1. reparse file
2. update index
3. never overwrite source based solely on stale index data.
