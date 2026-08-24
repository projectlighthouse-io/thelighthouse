# content-fixture

A fake content repo, for tests.

The real one is private, is called ohara, and lives at `CONTENT_PATH` — see
`crates/api/src/ohara`. Nothing here is a lesson anybody reads; it exists so
the loader has a tree to walk and the paywall split has something to cut.

**This one loads cleanly, end to end.** Anything deliberately wrong belongs in
`content-fixture-broken/` instead. The catalogue walks the whole repo and
refuses all of it if any book is bad, so a mistake planted here would fail
every test about something else.

Keep it thin. If a test needs a new shape, add the smallest file that has it.
