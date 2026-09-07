-- An article is about more than one thing.
--
-- `articles.category` held exactly one topic, and that was wrong the first
-- time somebody wrote about running Go in a container: the piece is about both
-- and had to pick. This replaces it with a join table, so an article carries
-- as many topics as it is actually about.
--
-- **No `topics` table.** The list is eight slugs that change about once a year
-- and it is already hardcoded in `articles::payload::CATEGORIES` — the api
-- refuses anything not on it. A table of ids would add a join to every read
-- and a second place for the list to be wrong, to buy referential integrity
-- over a set that is a constant in the binary. The slug is the id.
--
-- `ON DELETE CASCADE`, unlike `articles.user_id` and the other tables here.
-- The reasoning that keeps a membership when a user goes does not apply: a
-- topic row is not a record of anything, it is an edge, and an edge to a
-- deleted article is garbage rather than history.

CREATE TABLE article_topics (
    article_id BIGINT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    topic VARCHAR(32) NOT NULL,

    -- The pair is the identity: an article cannot be tagged `go` twice, and
    -- the api does not have to dedupe defensively before writing.
    PRIMARY KEY (article_id, topic)
);

-- Carry the existing rows across before the column goes. Every article had a
-- category and it was `NOT NULL`, so this loses nothing.
INSERT INTO article_topics (article_id, topic)
SELECT id, category FROM articles;

ALTER TABLE articles DROP COLUMN category;

-- `/blog?topic=rust` reads by topic and wants the article ids for one of them.
-- The primary key above already indexes (article_id, topic), which serves the
-- other direction — "what is this article about" — but cannot answer this one.
CREATE INDEX article_topics_topic_index ON article_topics (topic);

-- The category index on `articles` went with the column it indexed.
DROP INDEX IF EXISTS articles_live_by_category_index;
