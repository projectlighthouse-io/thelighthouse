# fixture

A fake content repo, for tests.

The real one is private, is called ohara, and lives at `CONTENT_PATH` — see
`crates/ohara`. Nothing here is a lesson anybody reads; it exists so
the loader has a tree to walk and the paywall split has something to cut.

```text
  books/          the good books — with projects/, this tree loads cleanly, end to end
  projects/       one project and its tasks
  mislabelled/    a content root where things are wrong on purpose
  duplicate-ids/  a content root where two lessons share an id
  drafts/         a content root whose only book is a draft, and a draft project
```

It is also what CI builds the image against, in place of ohara:
`make image CONTENT_PATH=fixture`.

`mislabelled/` is a sibling of `books/` and not a book inside it, which is the
whole point: the catalogue walks a repo and refuses all of it when one book is
malformed, so a mistake planted among the good books would fail every test that
only wanted a tree to walk. Out here it is reachable as `fixture::broken()` and
invisible to everything else. `duplicate-ids/` and `drafts/` are siblings for
the same reason, reached as `fixture::duplicate_ids()` and `fixture::drafts()`.

Keep it thin. If a test needs a new shape, add the smallest file that has it.
