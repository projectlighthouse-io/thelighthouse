-- courses become books.
--
-- The site has called them books everywhere a reader can see for a long time —
-- /books, the shelf, the covers — while the schema still said courses. This is
-- the rename that makes the database agree with the product.
--
-- **This breaks the laravel app.** Every model, query and migration there binds
-- to `courses`, `course_translations` and `course_id`, so the moment this runs
-- that stack starts erroring on anything book-shaped. It is safe on a database
-- only the rebuild reads, and on production it is a cutover step — not
-- something to run while laravel is still serving traffic.
--
-- Renames only. No data moves, no column types change, and every row keeps its
-- id, so this is fast and reversible by hand if it has to be. sqlx runs each
-- migration in a transaction, so it either all lands or none of it does.
--
-- Postgres renames less than people expect: `ALTER TABLE ... RENAME TO` moves
-- the table and nothing else. The sequence behind its id, the indexes, and the
-- constraint names all keep saying "course" unless renamed too — which is how
-- you end up with a `books` table whose primary key is `courses_pkey` and a
-- schema that half-remembers a word nobody uses. All 28 objects are below.

-- tables

ALTER TABLE courses RENAME TO books;
ALTER TABLE course_translations RENAME TO book_translations;

-- columns
--
-- `projects` is in here too. It carries both a course_id and its own ordering
-- column, which is easy to miss when grepping for the foreign key alone.

ALTER TABLE book_translations RENAME COLUMN course_id TO book_id;
ALTER TABLE lessons RENAME COLUMN course_id TO book_id;
ALTER TABLE projects RENAME COLUMN course_id TO book_id;
ALTER TABLE projects RENAME COLUMN course_sort_order TO book_sort_order;

-- sequences
--
-- Owned by the id columns, and not carried along by the table rename.

ALTER SEQUENCE courses_id_seq RENAME TO books_id_seq;
ALTER SEQUENCE course_translations_id_seq RENAME TO book_translations_id_seq;

-- constraints
--
-- Renaming a primary key or unique constraint renames the index behind it, so
-- these must not be repeated in the index section below.

ALTER TABLE books RENAME CONSTRAINT courses_pkey TO books_pkey;
ALTER TABLE books RENAME CONSTRAINT courses_slug_unique TO books_slug_unique;

ALTER TABLE book_translations
    RENAME CONSTRAINT course_translations_pkey TO book_translations_pkey;
ALTER TABLE book_translations
    RENAME CONSTRAINT course_translations_course_id_foreign TO book_translations_book_id_foreign;
ALTER TABLE book_translations
    RENAME CONSTRAINT course_translations_course_id_locale_unique TO book_translations_book_id_locale_unique;

ALTER TABLE lessons
    RENAME CONSTRAINT lessons_course_id_foreign TO lessons_book_id_foreign;
ALTER TABLE lessons
    RENAME CONSTRAINT lessons_course_id_slug_unique TO lessons_book_id_slug_unique;

ALTER TABLE projects
    RENAME CONSTRAINT projects_course_id_foreign TO projects_book_id_foreign;

-- indexes
--
-- Only the ones no constraint owns; the rest were renamed above.

ALTER INDEX courses_is_published_index RENAME TO books_is_published_index;
ALTER INDEX courses_published_at_index RENAME TO books_published_at_index;
ALTER INDEX courses_slug_index RENAME TO books_slug_index;

ALTER INDEX course_translations_locale_index RENAME TO book_translations_locale_index;

ALTER INDEX lessons_course_id_chapter_id_sort_order_index
    RENAME TO lessons_book_id_chapter_id_sort_order_index;
ALTER INDEX lessons_course_id_index RENAME TO lessons_book_id_index;
ALTER INDEX lessons_course_id_slug_index RENAME TO lessons_book_id_slug_index;

ALTER INDEX projects_course_id_index RENAME TO projects_book_id_index;
