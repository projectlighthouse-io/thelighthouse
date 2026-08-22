-- notes: drop the two xpath columns.
--
-- `xpath_start` and `xpath_end` are written by the browser when a note is
-- saved, stored, round-tripped back out, and never read by anything that
-- decides where a note goes. The anchor that does the work is the character
-- offset pair over the lesson body — `start_offset` and `end_offset`, which
-- stay.
--
-- They are not merely unused, they are wrong. The generator emits paths of the
-- form `text()[n]`, where `n` counts same-tag *element* siblings rather than
-- text nodes, so the expression does not identify the node it was built from.
-- Carrying that into the rebuild would mean porting a fallback that cannot
-- fall back.
--
-- Dropping rather than leaving them: a nullable column nobody reads is a column
-- somebody eventually reads, and this one would hand them an anchor that misses.
--
-- Not reversible. The values are the only copy, and there is no way to
-- reconstruct an xpath from an offset. That is acceptable because nothing has
-- ever consumed them — see `docs/rebuild.md`, phase 6.

ALTER TABLE notes DROP COLUMN xpath_start;
ALTER TABLE notes DROP COLUMN xpath_end;
