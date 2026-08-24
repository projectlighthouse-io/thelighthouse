# fixture

A fake content repo, for tests.

The real one is private, is called ohara, and lives at `CONTENT_PATH` — see
`crates/api/src/ohara`. Nothing here is a lesson anybody reads; it exists so
the loader has a tree to walk and the paywall split has something to cut.

```text
  books/        the good books — this tree loads cleanly, end to end
  pricing/      ppp.yaml, with rates chosen to be checkable by hand
  mislabelled/  a content root where things are wrong on purpose
```

`mislabelled/` is a sibling of `books/` and not a book inside it, which is the
whole point: the catalogue walks a repo and refuses all of it when one book is
malformed, so a mistake planted among the good books would fail every test that
only wanted a tree to walk. Out here it is reachable as `fixture::broken()` and
invisible to everything else.

Keep it thin. If a test needs a new shape, add the smallest file that has it.
