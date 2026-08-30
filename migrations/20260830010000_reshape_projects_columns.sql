-- projects: everything a reader reads leaves the table, and `is_published`
-- becomes the same `status` smallint books took.
--
-- The rule is the one `crates/content/src/sync.rs` states for books and
-- lessons: **the database is not where content is read from**. A project's
-- headline, pitch, features, difficulty and images are served from ohara's
-- `project.yaml`, so a column holding a second copy is a copy that goes stale
-- the first time the yaml is edited without a sync. Its name and short
-- description already left, with `project_translations`.
--
-- What survives is what other tables have to point at, plus what the sync
-- itself writes: an id, a slug, and whether the project is published.
--
--   status   0 draft, 1 published
--
-- Mapped by the same rust enum books use — see the COMMENT below.
--
-- Dropped, and why each is safe to lose:
--
--   headline, long_description         ohara. The api serves them from the
--   markdown, features, images         catalogue, never from a row.
--   metadata, difficulty
--
--   runner_image, unlock_mode          ohara, and read by luxctl or by the
--   is_challenge, show_tasks           api's own gating, both of which have
--   is_featured, featured_order        the catalogue in memory already.
--
--   book_id, book_sort_order           the link to a book is content: ohara's
--                                      `related_book_slug` states it. A
--                                      foreign key here would be a second
--                                      answer that can disagree with the yaml.
--
--   published_at                       `status` is the only published/not
--                                      question the rebuild asks.
--   uuid                               the id becomes one — see the migration
--                                      that follows this pair.
--
-- No data is lost that ohara does not already hold: every one of these
-- projects was converted into `projects/<slug>/project.yaml` before this ran.

-- status, backfilled from is_published exactly as books was.

ALTER TABLE projects ADD COLUMN status smallint NOT NULL DEFAULT 0;

UPDATE projects SET status = 1 WHERE is_published;

ALTER TABLE projects DROP COLUMN is_published;

CREATE INDEX projects_status_index ON projects (status);

ALTER TABLE projects ADD CONSTRAINT projects_status_check CHECK (status IN (0, 1));

-- The default stays, for the reason books gives: a project created without
-- saying otherwise should be a draft, because the other default publishes
-- something by accident.
COMMENT ON COLUMN projects.status IS
    '0 draft, 1 published. Mapped by content::Status in crates/api; change both together.';

-- what ohara serves

ALTER TABLE projects DROP COLUMN headline;
ALTER TABLE projects DROP COLUMN long_description;
ALTER TABLE projects DROP COLUMN markdown;
ALTER TABLE projects DROP COLUMN features;
ALTER TABLE projects DROP COLUMN images;
ALTER TABLE projects DROP COLUMN metadata;
ALTER TABLE projects DROP COLUMN difficulty;
ALTER TABLE projects DROP COLUMN runner_image;
ALTER TABLE projects DROP COLUMN unlock_mode;
ALTER TABLE projects DROP COLUMN is_challenge;
ALTER TABLE projects DROP COLUMN show_tasks;
ALTER TABLE projects DROP COLUMN is_featured;
ALTER TABLE projects DROP COLUMN featured_order;

-- the link to a book, which ohara states

ALTER TABLE projects DROP COLUMN book_id;
ALTER TABLE projects DROP COLUMN book_sort_order;

-- retired
--
-- `lab_slug` is not here: 20260828040000 took it with the rest of the lab
-- infrastructure.

ALTER TABLE projects DROP COLUMN published_at;
ALTER TABLE projects DROP COLUMN uuid;
