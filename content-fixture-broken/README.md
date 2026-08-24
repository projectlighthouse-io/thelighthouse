# content-fixture-broken

A content repo where things are wrong on purpose.

Its own root and not a corner of `content-fixture/`, because the catalogue
walks a whole repo and refuses all of it when one book is malformed — so a
planted mistake in the good fixture would fail every test that only wanted a
tree to walk.

Reached from tests as `fixture::broken()`.

| what is wrong | who checks it |
|---|---|
| `mislabelled/book.yaml` says `not-mislabelled` | `book.rs`, `catalog.rs` |
| `01-right-name/lesson.yaml` says `wrong-name` | `lesson.rs` |
