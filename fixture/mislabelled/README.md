# fixture/mislabelled

A content root where things are wrong on purpose.

Beside `fixture/books/` rather than inside it, so nothing that walks the good
books ever meets it. Reached from tests as `fixture::broken()`.

| what is wrong | who checks it |
|---|---|
| `books/mislabelled/book.yaml` says `not-mislabelled` | `book.rs`, `catalog.rs` |
| `01-right-name/lesson.yaml` says `wrong-name` | `lesson.rs` |
| `projects/fixture-project/tasks/03-misnamed/task.yaml` says `something-else` | `task.rs` |
