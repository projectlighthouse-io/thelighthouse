-- books: a permanent identity that a rename cannot break.
--
-- `slug` is the only identifier a book has today, in the yaml and in this
-- table, and it is editable prose — part of the url, tuned for search. That
-- makes it the wrong thing for another table to point at. An entitlement keyed
-- on `book_slug` survives exactly as long as nobody renames the book; the day
-- somebody does, every reader who paid for it silently loses access, and
-- nothing fails loudly enough to notice.
--
-- `books.id` alone does not fix it. A sync that matches rows on slug treats a
-- renamed book as a new one, inserts a second row with a new id, and the
-- entitlement still points at the row nobody reads any more. The stable key has
-- to travel with the content, so it lives in `book.yaml` and this column is
-- where it lands.
--
-- The default is for the backfill and nothing else. Production carries real
-- books rows from laravel, so the column cannot arrive NOT NULL without one;
-- immediately after, the default is dropped, so every future insert has to
-- supply a uuid the rust side generated. A column that mints its own identity
-- on insert is a column that hands out an id the content repo has never seen.
--
-- `gen_random_uuid()` is built in from postgres 13; no pgcrypto needed.

ALTER TABLE books
    ADD COLUMN uuid uuid NOT NULL DEFAULT gen_random_uuid();

ALTER TABLE books
    ALTER COLUMN uuid DROP DEFAULT;

-- Two entitlements pointing at one book is a bug in the sync, not a state to
-- represent.
ALTER TABLE books
    ADD CONSTRAINT books_uuid_unique UNIQUE (uuid);
