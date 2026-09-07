-- The line under the title.
--
-- Substack calls it a subtitle and it is doing real work: it is what a reader
-- decides from on the listing, and what search engines and link previews show
-- under the headline. Until now that line was machine-made — the first couple
-- of sentences of the body, cut at a word — which is a summary of how an
-- article *starts* rather than what it is about.
--
-- Required, so every article has one. `varchar(200)` because postgres counts
-- that in characters and the api's limit is the same number, so the two agree
-- rather than one truncating what the other accepted.
--
-- Added nullable, backfilled, then made `NOT NULL`: a `NOT NULL` column with
-- no default cannot be added to a table that already has rows, and inventing a
-- default would leave every existing article sharing one sentence.

ALTER TABLE articles ADD COLUMN subtitle VARCHAR(200);

-- A machine-made line for the articles written before the column existed —
-- the same shape of thing the listing was deriving anyway, now frozen so it
-- can be edited rather than recomputed. Authors can rewrite it; that is the
-- point of the column.
--
-- Fenced blocks go first, so an article opening with code does not get a
-- subtitle made of yaml. Then the markdown that would otherwise show as
-- punctuation, then whitespace collapsed, then cut to fit.
UPDATE articles
SET subtitle = NULLIF(
    btrim(
        left(
            regexp_replace(
                regexp_replace(
                    regexp_replace(body, '```[\s\S]*?```', ' ', 'g'),
                    '^[ \t]*[#>]+[ \t]*', '', 'ng'
                ),
                '\s+', ' ', 'g'
            ),
            200
        )
    ),
    ''
)
WHERE subtitle IS NULL;

-- Anything the strip above emptied — an article that is nothing but a code
-- block, say — still needs a value, and a visible placeholder is better than a
-- blank line nobody can see is missing.
UPDATE articles
SET subtitle = title
WHERE subtitle IS NULL;

ALTER TABLE articles ALTER COLUMN subtitle SET NOT NULL;
