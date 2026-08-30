-- The three-table progress model goes, before the migration that rebuilds it
-- as one log.
--
-- Laravel kept `user_project_attempts` (a row per submission),
-- `user_task_progress` (a row per task holding the current status) and
-- `project_attempt_groups` (a row per restart, which the other two point at to
-- say which run they belong to). The second is derivable from the first, and
-- keeping both means two writes per submission and two answers whenever they
-- disagree — `GetTask` in the laravel app carries a branch for progress rows
-- whose `attempt_group_id` is null, which is exactly that disagreement showing
-- up as code.
--
-- They are dropped rather than migrated. In this database they hold nothing:
-- the rebuild has never written a task attempt, and the laravel rows live in
-- the database laravel still owns. The guard below refuses to run if that ever
-- stops being true, because dropping a reader's completed work to tidy a
-- schema is not a trade anyone would make.

DO $$
DECLARE
    populated text;
BEGIN
    SELECT string_agg(name, ', ')
      INTO populated
      FROM (
          SELECT 'user_project_attempts' AS name
           WHERE EXISTS (SELECT 1 FROM user_project_attempts)
          UNION ALL
          SELECT 'user_task_progress'
           WHERE EXISTS (SELECT 1 FROM user_task_progress)
          UNION ALL
          SELECT 'project_attempt_groups'
           WHERE EXISTS (SELECT 1 FROM project_attempt_groups)
      ) AS populated_tables;

    IF populated IS NOT NULL THEN
        RAISE EXCEPTION
            'these hold rows and this migration would destroy them: %. '
            'Write the backfill into task_attempts before applying it.', populated;
    END IF;
END $$;

DROP TABLE user_project_attempts;
DROP TABLE user_task_progress;
DROP TABLE project_attempt_groups;
