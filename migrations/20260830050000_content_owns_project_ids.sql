-- projects, tasks and task_hints are identified by the uuid the content repo
-- writes, exactly as books and lessons already are.
--
-- The reasoning is `20260828020000_content_owns_its_ids.sql`, and it is not
-- repeated here beyond the one sentence that matters: an integer has to be
-- assigned by whoever can see all the others, which means the yaml cannot name
-- a task until after the first sync — and until it can, a second sync inserts
-- the task twice. A uuid is minted in the file, before the row exists.
--
-- These three were left out of that migration because nothing read them yet.
-- They are in now because `.tinker/ohara-project-import.php` has written an
-- `id:` into every `project.yaml`, `task.yaml` and hint in ohara, and those are
-- the ids the sync will insert.
--
-- What points at them: `user_unlocked_hints.task_hint_id` — and, after the
-- migration that follows, `task_attempts.task_id`. The laravel progress tables
-- that also pointed here were dropped one migration ago, which is why this one
-- has so little to repoint.
--
-- Safe as a plain type change only while these tables are empty, so the guard
-- below insists on it rather than trusting the note.

DO $$
DECLARE
    populated text;
BEGIN
    SELECT string_agg(name, ', ')
      INTO populated
      FROM (
          SELECT 'projects' AS name WHERE EXISTS (SELECT 1 FROM projects)
          UNION ALL
          SELECT 'tasks' WHERE EXISTS (SELECT 1 FROM tasks)
          UNION ALL
          SELECT 'task_hints' WHERE EXISTS (SELECT 1 FROM task_hints)
          UNION ALL
          SELECT 'user_unlocked_hints' WHERE EXISTS (SELECT 1 FROM user_unlocked_hints)
      ) AS populated_tables;

    IF populated IS NOT NULL THEN
        RAISE EXCEPTION
            'these hold rows, so this is a data migration rather than a type '
            'change: %. The ids in ohara are the ones to map onto.', populated;
    END IF;
END $$;

-- A referencing column cannot change type while a constraint ties it to the
-- old one.
ALTER TABLE tasks DROP CONSTRAINT tasks_project_id_foreign;
ALTER TABLE task_hints DROP CONSTRAINT task_hints_task_id_foreign;
ALTER TABLE user_unlocked_hints DROP CONSTRAINT user_unlocked_hints_task_hint_id_foreign;

ALTER TABLE projects
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE uuid USING gen_random_uuid();

ALTER TABLE tasks
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE uuid USING gen_random_uuid(),
    ALTER COLUMN project_id TYPE uuid USING gen_random_uuid();

ALTER TABLE task_hints
    ALTER COLUMN id DROP DEFAULT,
    ALTER COLUMN id TYPE uuid USING gen_random_uuid(),
    ALTER COLUMN task_id TYPE uuid USING gen_random_uuid();

ALTER TABLE user_unlocked_hints
    ALTER COLUMN task_hint_id TYPE uuid USING gen_random_uuid();

-- The sequences own nothing now, and a sequence beside a uuid column is an
-- invitation to reach for it.
DROP SEQUENCE IF EXISTS projects_id_seq;
DROP SEQUENCE IF EXISTS tasks_id_seq;
DROP SEQUENCE IF EXISTS task_hints_id_seq;

-- Restored without ON DELETE, for the reason the books migration gives: a
-- resync that dropped a task for a moment would take a reader's unlocks with
-- it and report success. Refusing the delete is the right answer to "this task
-- has work recorded against it".
ALTER TABLE tasks
    ADD CONSTRAINT tasks_project_id_foreign
    FOREIGN KEY (project_id) REFERENCES projects(id);

ALTER TABLE task_hints
    ADD CONSTRAINT task_hints_task_id_foreign
    FOREIGN KEY (task_id) REFERENCES tasks(id);

ALTER TABLE user_unlocked_hints
    ADD CONSTRAINT user_unlocked_hints_task_hint_id_foreign
    FOREIGN KEY (task_hint_id) REFERENCES task_hints(id);
