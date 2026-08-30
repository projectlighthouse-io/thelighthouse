-- tasks and task_hints: the same rule, applied one level down.
--
-- A task's title, prose, points, scoring ladder and hint text are ohara's —
-- `projects/<project>/tasks/<order>-<slug>/task.yaml` and the `task.md` beside
-- it. What is left is the row `task_attempts` and `user_unlocked_hints` need
-- something to point at: an id, which project and task it belongs to, and the
-- order it is read in.
--
-- The largest thing leaving is `tasks.blueprint`, which held **the entire .bp
-- file, once per task**. Eighteen tasks in one project meant eighteen copies
-- of the same 46 KB — and that column was the api's answer when luxctl asked
-- for a blueprint. The api now reads the one file ohara holds and serves it,
-- so the copies have no reader.
--
-- Dropped from tasks, and why:
--
--   blueprint                          above. One file, in ohara, read once.
--   points, scores                     ohara. The ledger stores what a reader
--   abandoned_deduction                actually earned; what a task is *worth*
--                                      is content and moves when it is edited.
--   visibility_level, is_free          ohara. Which tasks are paid is a
--                                      content decision; entitlement is
--                                      answered against the project's related
--                                      book, not against a column here.
--   input_type                         ohara, and read by luxctl.
--   metadata, uuid                     the id becomes the uuid.
--
-- Dropped from task_hints:
--
--   text, unlock_criteria              ohara. Hint prose in a row is prose no
--   points_deduction                   entitlement check guards, and the
--                                      deduction a reader paid is recorded on
--                                      the unlock itself.
--   uuid                               as above.
--
-- `user_unlocked_hints.task_id` goes too. A hint belongs to exactly one task
-- and ohara says which, so the column is a second answer that can disagree
-- with the first — and the api holds the catalogue in memory, where the hint
-- already knows its task. `task_hint_id` is the only pointer an unlock needs.
--
-- `sort_order` stays on both. It is the folder's number in ohara and could be
-- read from there, but it is also what every listing orders by, and ordering
-- in SQL beats ordering after the fact.

ALTER TABLE tasks DROP COLUMN blueprint;
ALTER TABLE tasks DROP COLUMN points;
ALTER TABLE tasks DROP COLUMN scores;
ALTER TABLE tasks DROP COLUMN abandoned_deduction;
ALTER TABLE tasks DROP COLUMN visibility_level;
ALTER TABLE tasks DROP COLUMN is_free;
ALTER TABLE tasks DROP COLUMN input_type;
ALTER TABLE tasks DROP COLUMN metadata;
ALTER TABLE tasks DROP COLUMN uuid;

ALTER TABLE task_hints DROP COLUMN text;
ALTER TABLE task_hints DROP COLUMN unlock_criteria;
ALTER TABLE task_hints DROP COLUMN points_deduction;
ALTER TABLE task_hints DROP COLUMN uuid;

-- Dropping the column takes user_unlocked_hints_task_id_foreign with it.
ALTER TABLE user_unlocked_hints DROP COLUMN task_id;
