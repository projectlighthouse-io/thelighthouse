# fixture/duplicate-ids

A content root where two lessons claim the same id.

Its own root, beside `fixture/books/` rather than inside it, for the reason
`fixture/mislabelled/` gives: a catalogue walks the whole repo and refuses all
of it when one file is wrong, so a mistake planted among the good books would
fail every test that only wanted a tree to walk.

Reached from tests as `fixture::duplicate_ids()`.

| what is wrong | who checks it |
|---|---|
| `books/twice/lessons/02-second` reuses `01-first`'s id | `catalog.rs` |
