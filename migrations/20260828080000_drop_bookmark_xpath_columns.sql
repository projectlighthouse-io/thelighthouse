-- lesson_bookmarks: drop the two xpath columns.
--
-- The same decision `20260822220000_drop_note_xpath_columns.sql` made for
-- notes, for the same reason and against the same generator: the browser emits
-- paths of the form `text()[n]`, where `n` counts same-tag *element* siblings
-- rather than text nodes, so the expression does not identify the node it was
-- built from. A fallback that cannot fall back is worse than none.
--
-- What anchors a bookmark is the character offset pair over the rendered
-- lesson body — `start_offset` and `end_offset`, both `NOT NULL`, which stay.
--
-- Not reversible, and acceptable for the reason they were droppable at all:
-- nothing has ever read them. See `docs/rebuild.md`, phase 6.

ALTER TABLE lesson_bookmarks DROP COLUMN xpath_start;
ALTER TABLE lesson_bookmarks DROP COLUMN xpath_end;
