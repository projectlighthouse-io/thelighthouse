-- An author putting their own article away.
--
-- Not a delete: the article, its slug and its topics stay, and the author can
-- bring it back. While archived it is off the public listing and its page
-- answers 404 to everybody but the author, who still sees it on their own
-- shelf. The same shape as `taken_down_at`, and deliberately a separate column:
-- a takedown is a moderator's, cannot be undone by the author and carries a
-- reason; an archive is the author's own and needs none.
--
-- Nullable with no default: every existing article is live, which is NULL.

ALTER TABLE articles ADD COLUMN archived_at TIMESTAMP(0);
