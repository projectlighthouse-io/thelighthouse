-- books and lessons: drop `published_at`.
--
-- `status` is what every decision is made from — the catalogue holds published
-- books and published lessons and nothing else, so by the time a row or a
-- request reaches anything, "is this live" has already been answered. Nothing
-- reads `published_at`, and nothing was going to: it never reached a view, so
-- it never reached the wire either.
--
-- Not dropped for being merely unused. A nullable column nobody reads is a
-- column somebody eventually reads, and this one would be empty — the sync
-- writes what the yaml carries, and the model no longer carries this. A date
-- column that silently holds NULL for every row is worse than no column,
-- because the first thing built on it looks like it works.
--
-- The dates are not lost. `book.yaml` and every `lesson.yaml` still carry
-- `published_at`; serde ignores what the struct does not name. If lesson SEO
-- ever wants `datePublished` and a real sitemap `lastmod` — and it is the
-- ranking surface, so it might — the column comes back and the values are
-- still sitting in the content repo.
--
-- `projects.published_at` is left alone. Projects are not synced from ohara
-- and this migration has no business reaching into them.
--
-- The three indexes go with the columns; postgres drops them automatically.

ALTER TABLE books DROP COLUMN published_at;
ALTER TABLE lessons DROP COLUMN published_at;
