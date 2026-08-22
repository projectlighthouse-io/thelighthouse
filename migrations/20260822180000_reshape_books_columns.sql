-- books: two booleans-and-strings become two small integers, and three columns
-- the rebuild does not carry go away.
--
-- The two new columns are stored as smallint and given names in rust, in
-- `crates/api/src/content.rs`. Integers rather than text because the set of
-- values is closed and known at compile time: a typo in a varchar column is a
-- row that reads as neither published nor draft and is silently absent from
-- every listing, whereas a smallint outside its CHECK cannot be written at all.
-- The rust enum and the CHECK below have to be changed together — the COMMENTs
-- on each column name the type to change alongside it.
--
--   status   0 draft, 1 published
--   tier     0 foundation, 1 intermediate, 2 advanced
--
-- Dropped: `is_published`, replaced by `status`. A draft is written but not for
-- readers: a local run shows drafts so they can be read while being written,
-- production does not. A boolean cannot say that.
-- `lab_slug` and `details_component`, which belong to the retired lab
-- infrastructure and a laravel component name respectively — neither means
-- anything in the rebuild.
-- `required_tier`, which is not what the new `tier` is. See below.

-- status
--
-- Backfilled from is_published, which is a faithful mapping: everything that
-- was published becomes published, everything else becomes a draft.

ALTER TABLE books ADD COLUMN status smallint NOT NULL DEFAULT 0;

UPDATE books SET status = 1 WHERE is_published;

-- Dropping the column drops books_is_published_index with it, so the index is
-- recreated against the column that replaced it. Listings filter on this.
ALTER TABLE books DROP COLUMN is_published;

CREATE INDEX books_status_index ON books (status);

ALTER TABLE books ADD CONSTRAINT books_status_check CHECK (status IN (0, 1));

-- The default stays. A book that is created without saying otherwise should be
-- a draft — the failure mode of the other default is publishing something by
-- accident.
COMMENT ON COLUMN books.status IS
    '0 draft, 1 published. Mapped by content::Status in crates/api; change both together.';

-- tier
--
-- `required_tier` answered "which subscription unlocks this". `tier` answers
-- "how hard is this". Those are different questions, and the column is only a
-- faithful backfill because of what is actually in it: every row is 'voyage',
-- and voyage becomes foundation. So the whole table lands on foundation, which
-- is a real mapping rather than a shrug.
--
-- Verified before writing this: 11 of 11 rows are 'voyage'. The guard below
-- re-checks it at apply time rather than trusting that snapshot, because this
-- migration runs against production much later than it was written and a value
-- added in between would otherwise be silently flattened to foundation.

DO $$
DECLARE
    unexpected text;
BEGIN
    SELECT string_agg(DISTINCT required_tier, ', ')
      INTO unexpected
      FROM books
     WHERE required_tier IS DISTINCT FROM 'voyage';

    IF unexpected IS NOT NULL THEN
        RAISE EXCEPTION
            'books.required_tier holds values this migration has no mapping for: %. '
            'Decide what they become and amend the backfill.', unexpected;
    END IF;
END $$;

-- DEFAULT 0 fills every existing row in one pass: voyage, and therefore
-- foundation.
ALTER TABLE books ADD COLUMN tier smallint NOT NULL DEFAULT 0;

-- The default is then dropped, unlike status. There is no safe tier to assume
-- for a *new* book, and one quietly labelled foundation is worse than an insert
-- that fails and asks. The default existed only for the backfill above.
ALTER TABLE books ALTER COLUMN tier DROP DEFAULT;

ALTER TABLE books ADD CONSTRAINT books_tier_check CHECK (tier IN (0, 1, 2));

COMMENT ON COLUMN books.tier IS
    '0 foundation, 1 intermediate, 2 advanced. Mapped by content::Tier in crates/api; change both together.';

-- Entitlement does not go with it. Deciding what a subscription unlocks belongs
-- to the entitlements table in a later migration, and `tier` must never be
-- consulted for access — it is a statement about difficulty, not about payment.
ALTER TABLE books DROP COLUMN required_tier;

-- retired columns

ALTER TABLE books DROP COLUMN lab_slug;
ALTER TABLE books DROP COLUMN details_component;
