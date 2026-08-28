-- entitlements: one row per book a reader may read in full.
--
-- Keyed on the book's id, not on its slug.
--
-- Not the slug, because a slug is url prose and gets retuned; an entitlement
-- pointing at one dies silently the day somebody renames a book, and the only
-- symptom is a reader who paid being shown a buy button.
--
-- The id lives in `book.yaml` and travels with the content, so it is already
-- in memory by the time the check runs and this costs no join. It owes laravel
-- nothing: the cutover carries users, subscriptions and notes across and
-- nothing else, so the books arrive fresh and the content repo is free to
-- number them from one.
--
-- No foreign key to `books`, and that is deliberate rather than an oversight.
-- Nothing populates that table — the catalogue is parsed from the content repo
-- at boot and held in memory. The Makefile says it outright: "There is no step
-- that writes content into a database." A foreign key to a table that stays
-- empty would make every entitlement unwritable.
--
-- The trade is that the database cannot enforce that the book exists. It
-- cannot anyway: the books are files.
--
-- `source` records why the reader has this — bought it, got it in a bundle,
-- was granted it. Nothing writes entitlements yet, so nothing reads this yet
-- either; it is here because backfilling the reason for a purchase after the
-- fact is guesswork, and the column is free until then.
--
-- `expires_at` null means forever, which is what a purchase is. A grant with
-- an end date is the case that needs the column.
--
-- No ON DELETE on the user key. Cascading would make deleting a user silently
-- destroy the record of what they bought, and that record is the thing an
-- argument about a refund is settled with. Refusing the delete says there is
-- something to deal with first.

CREATE TABLE entitlements (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    book_id uuid NOT NULL,
    source SMALLINT NOT NULL DEFAULT 0,
    granted_at TIMESTAMP(0) NOT NULL DEFAULT now(),
    expires_at TIMESTAMP(0),

    CONSTRAINT entitlements_source_check CHECK (source = ANY (ARRAY[0, 1, 2]))
);

-- Owning a book twice is not a state to represent. A bundle that grants a book
-- the reader already bought updates the row it finds rather than adding one.
CREATE UNIQUE INDEX entitlements_user_book_unique
    ON entitlements (user_id, book_id);

-- The read path is always "this reader, this book", which the unique index
-- above already serves. This one is for the other direction — every book a
-- reader owns, for the dashboard.
CREATE INDEX entitlements_user_id_index ON entitlements (user_id);
