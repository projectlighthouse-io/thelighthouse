-- articles: prose a reader wrote, as markdown, shown at /blog.
--
-- Not ohara's. The content repo holds the books and the lessons, is private,
-- and is edited by whoever has push to it; this table holds what a signed-in
-- reader typed into a box. Two different things with two different trust
-- levels, so they do not share a store — an article can never become a lesson
-- by accident, and a content sync can never overwrite one.
--
-- `body` is the markdown as typed, never rendered html. Nuxt owns rendering
-- for its pages — `docs/rebuild.md`, route prefixes — and the api storing
-- rendered html would mean this table holding markup a reader supplied. It
-- also means fixing the renderer never needs a backfill.
--
-- `slug` is minted once from the title with a random suffix and never changes,
-- so retitling an article does not break a link somebody shared or a result
-- that was indexed. The suffix is what makes two articles called "Getting
-- started" both possible without a retry loop around the insert.
--
-- `category` is the topic, from a list the api hardcodes — `go`, `rust`,
-- `containers` and so on, in `articles::payload::CATEGORIES`. Not a table and
-- not a check constraint: the list is short, changes about once a year, and
-- adding a topic should be a line in Rust rather than a migration and a
-- deploy. It is also what lets a row be re-categorised by hand without the
-- database arguing about a value the api has not shipped yet.
--
-- `taken_down_at` and `taken_down_reason` are the whole moderation story.
-- Null is live, which is why nothing here has a `status` column and no article
-- waits for approval: posting publishes. Nothing in the api writes these two —
-- taking an article down is an `UPDATE` run by hand, deliberately, because it
-- happens rarely and an endpoint for it would be a permission system, a gate
-- and a screen for something a `psql` session already does. The reason is what
-- the author is shown, so a takedown is never a page that silently vanished.
--
-- One state, not two. "Paused" and "taken down" differ in what is meant by it,
-- not in what the site does: the article stops being served and the author is
-- told why. Restoring is clearing both columns.
--
-- No ON DELETE on the user key, matching memberships and entitlements.
-- Cascading would make deleting a user silently destroy prose other people may
-- have linked to.

CREATE TABLE articles (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    slug VARCHAR(255) NOT NULL,
    title VARCHAR(200) NOT NULL,
    category VARCHAR(32) NOT NULL,
    body TEXT NOT NULL,
    taken_down_at TIMESTAMP(0),
    taken_down_reason TEXT,
    created_at TIMESTAMP(0) NOT NULL DEFAULT now(),
    updated_at TIMESTAMP(0) NOT NULL DEFAULT now(),

    -- A takedown is both columns or neither. Without this an `UPDATE` typed in
    -- a hurry can hide an article and leave its author with no reason, which
    -- is the one thing the reason column exists to prevent.
    CONSTRAINT articles_takedown_has_a_reason CHECK (
        (taken_down_at IS NULL AND taken_down_reason IS NULL)
        OR (taken_down_at IS NOT NULL AND taken_down_reason IS NOT NULL)
    )
);

-- The url. Unique because it is the only way an article is addressed.
CREATE UNIQUE INDEX articles_slug_unique ON articles (slug);

-- The /blog listing: live articles, newest first. Partial, because a taken
-- down article is never in that listing and there is no reason to index it for
-- one.
CREATE INDEX articles_live_index
    ON articles (created_at DESC, id DESC)
    WHERE taken_down_at IS NULL;

-- The same listing narrowed to one topic, which is the other way /blog is
-- read. Partial for the same reason, and `category` leads so the scan starts
-- from the rows being asked for rather than filtering the whole shelf.
CREATE INDEX articles_live_by_category_index
    ON articles (category, created_at DESC, id DESC)
    WHERE taken_down_at IS NULL;

-- "my articles", which includes the taken down ones and is the only place an
-- author reads the reason.
CREATE INDEX articles_user_id_index ON articles (user_id);
