# content-fixture

A fake content repo, for tests.

The real one is private and lives at `CONTENT_PATH` — see
`crates/api/src/content`. Nothing here is a lesson anybody reads; it exists so
the loader has a tree to walk and the paywall split has something to cut.

Keep it thin. If a test needs a new shape, add the smallest file that has it.
