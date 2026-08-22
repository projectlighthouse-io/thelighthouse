-- lessons: the same status treatment as books, and four columns dropped.
--
-- `book_id` is already named that — the rename migration did it — so nothing
-- here touches the foreign key.
--
--   status   0 draft, 1 published    (content::Status, shared with books)
--
-- Dropped:
--
--   is_published        replaced by status, same as books.
--   editor_version      which laravel editor wrote the body. The rebuild
--                       renders markdown from the content repo at build time
--                       and has one renderer, so there is no version to track.
--   is_content_locked   a second boolean gate beside is_published. Two flags
--                       that both mean "not readable" is how content ends up
--                       hidden for a reason nobody can name; entitlement
--                       decides who reads a paid body, and status decides
--                       whether the lesson exists for readers at all.
--   lab_slug            retired lab infrastructure, same as on books.
--   visibility_level    see the note below — this one is not free.

-- status

ALTER TABLE lessons ADD COLUMN status smallint NOT NULL DEFAULT 0;

UPDATE lessons SET status = 1 WHERE is_published;

-- Two indexes are on is_published and both go when the column does. The pair
-- is recreated against status because they are the shape every listing reads:
-- "what is published", and "what is published, newest first".
ALTER TABLE lessons DROP COLUMN is_published;

CREATE INDEX lessons_status_index ON lessons (status);
CREATE INDEX lessons_status_published_at_index ON lessons (status, published_at);

ALTER TABLE lessons ADD CONSTRAINT lessons_status_check CHECK (status IN (0, 1));

-- Default kept, as on books: a lesson created without saying otherwise is a
-- draft, because the other default publishes something by accident.
COMMENT ON COLUMN lessons.status IS
    '0 draft, 1 published. Mapped by content::Status in crates/api; change both together.';

-- retired columns

ALTER TABLE lessons DROP COLUMN editor_version;
ALTER TABLE lessons DROP COLUMN is_content_locked;
ALTER TABLE lessons DROP COLUMN lab_slug;

-- visibility_level
--
-- Dropped on instruction, and worth writing down what goes with it: this is an
-- int defaulting to 1, and on a platform with a paywall the likeliest reading
-- is how much of a lesson an unentitled reader sees. If that is what it meant,
-- the free/paid split now has to come from somewhere else.
--
-- docs/rebuild.md says it does: lesson bodies are split at a paywall marker in
-- the content repo and rendered into two fragments at build time, so which
-- fragment a reader gets is decided per request from their entitlement rather
-- than from a column. Under that design this column has no job. Recorded here
-- because dropping a column is easy and noticing later that it meant something
-- is not.

ALTER TABLE lessons DROP COLUMN visibility_level;
