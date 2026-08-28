-- books and lessons are identified by a uuid the content repo writes.
--
-- Both ids arrived from laravel as `bigserial`, which is right when the
-- database decides identity. It does not, any more. `book.yaml` and
-- `lesson.yaml` each carry an immutable id, and everything pointing at a book
-- or a lesson points at that — `entitlements.book_id`, and `notes.lesson_id`,
-- where a note follows character offsets into a body and landing on the wrong
-- lesson means landing on prose it was never written against.
--
-- A uuid rather than an integer, and the difference is who can mint one.
-- An integer has to be assigned by whoever can see all the others: either the
-- database hands it out on insert — in which case the yaml cannot carry it
-- until after the first sync, and there is a window where re-running the sync
-- inserts a second row — or it is written by hand and must not collide across
-- every book and lesson in the repo. A uuid is minted anywhere, by anyone,
-- with no coordination and no window. It can be written into the yaml before
-- the row exists, which is the whole point: the content repo is the authority
-- on identity, so it must be able to state it first.
--
-- The uuid *is* the primary key, not a second key beside a serial. Two
-- identities for one row means every table pointing at it has to pick, and the
-- one that picks the serial is back to waiting for an insert.
--
-- The restored keys carry no ON DELETE behaviour, and losing the CASCADE the
-- laravel schema had is the point. `notes.lesson_id` cascaded, so removing a
-- lesson row deleted every note written against it — and this is a table the
-- sync will be updating from files. A resync that dropped a lesson for a
-- moment would take a reader's notes with it and report success. Without the
-- clause the delete is refused, which is the right answer to "this lesson has
-- notes on it".
--
-- Safe as a plain type change because every one of these tables is empty:
-- nothing has been migrated yet, and the books have never been synced. The
-- day any of them holds a row this becomes a data migration instead.

-- Drop the foreign keys first: a referencing column cannot change type while a
-- constraint ties it to the old one.
ALTER TABLE lessons DROP CONSTRAINT lessons_book_id_foreign;
ALTER TABLE book_translations DROP CONSTRAINT book_translations_book_id_foreign;
ALTER TABLE projects DROP CONSTRAINT projects_book_id_foreign;
ALTER TABLE notes DROP CONSTRAINT notes_lesson_id_foreign;
ALTER TABLE lesson_bookmarks DROP CONSTRAINT lesson_bookmarks_lesson_id_foreign;
ALTER TABLE lesson_translations DROP CONSTRAINT lesson_translations_lesson_id_foreign;

ALTER TABLE books
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE uuid USING gen_random_uuid();

ALTER TABLE lessons
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE uuid USING gen_random_uuid(),
    ALTER COLUMN book_id TYPE uuid USING gen_random_uuid();

ALTER TABLE book_translations
    ALTER COLUMN book_id TYPE uuid USING gen_random_uuid();

-- Nullable, and stays that way: a project need not belong to a book.
ALTER TABLE projects
    ALTER COLUMN book_id TYPE uuid USING gen_random_uuid();

ALTER TABLE notes
    ALTER COLUMN lesson_id TYPE uuid USING gen_random_uuid();

ALTER TABLE lesson_bookmarks
    ALTER COLUMN lesson_id TYPE uuid USING gen_random_uuid();

ALTER TABLE lesson_translations
    ALTER COLUMN lesson_id TYPE uuid USING gen_random_uuid();

-- No foreign key ties this one to `lessons`, which is why enumerating the
-- references by constraint missed it. The column is a lesson id all the same,
-- and leaving it a bigint would make it the one place a lesson is named by a
-- number nothing issues any more. The missing constraint is left as it was:
-- adding one is a change to what the table permits, not to what it stores.
ALTER TABLE lesson_completions
    ALTER COLUMN lesson_id TYPE uuid USING gen_random_uuid();

-- The sequences own nothing now. Dropped rather than left orphaned: a sequence
-- beside a uuid column is an invitation to reach for it.
DROP SEQUENCE IF EXISTS books_id_seq;
DROP SEQUENCE IF EXISTS lessons_id_seq;

ALTER TABLE lessons
    ADD CONSTRAINT lessons_book_id_foreign
    FOREIGN KEY (book_id) REFERENCES books(id);

ALTER TABLE book_translations
    ADD CONSTRAINT book_translations_book_id_foreign
    FOREIGN KEY (book_id) REFERENCES books(id);

ALTER TABLE projects
    ADD CONSTRAINT projects_book_id_foreign
    FOREIGN KEY (book_id) REFERENCES books(id);

ALTER TABLE notes
    ADD CONSTRAINT notes_lesson_id_foreign
    FOREIGN KEY (lesson_id) REFERENCES lessons(id);

ALTER TABLE lesson_bookmarks
    ADD CONSTRAINT lesson_bookmarks_lesson_id_foreign
    FOREIGN KEY (lesson_id) REFERENCES lessons(id);

ALTER TABLE lesson_translations
    ADD CONSTRAINT lesson_translations_lesson_id_foreign
    FOREIGN KEY (lesson_id) REFERENCES lessons(id);
