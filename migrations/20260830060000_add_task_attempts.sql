-- Progress becomes one append-only log, plus a record of restarts.
--
-- `task_attempts` is what luxctl writes: a row per submission, never updated,
-- never deleted. Everything the three laravel tables computed is a query over
-- it —
--
--   status        no rows is "challenge awaits"; any `passed` row in the
--                 current run is completed; otherwise the newest row's
--                 outcome.
--   attempts      count.
--   started_at    the oldest row's `created_at`.
--   completed_at  the first `passed`.
--   points        the highest `points` among the passed rows, less whatever
--                 hints the reader unlocked.
--
-- Derived rather than stored because there is nothing to gain by storing it:
-- one reader, one project, tens of tasks, all of it behind an authenticated
-- endpoint that is not cached anyway. What there is to lose is the case where
-- the log and the summary disagree, which no amount of care in the writer
-- fully rules out.
--
-- `project_restarts` is what makes "which run" answerable without a nullable
-- foreign key on every attempt. A reader restarting a project appends a row;
-- the current run is `count(*) + 1`, and an attempt records the run it was
-- made in. The laravel model gave each run a row of its own that attempts
-- pointed at, which meant attempts written before restarts existed had a null
-- pointer and a special case in every reader — `GetTask` still carries it.
--
--   outcome   0 attempted, 1 passed, 2 failed, 3 abandoned
--
-- The first three are what luxctl submits today; `abandoned` is here because
-- ohara's `abandoned_deduction` implies it and a ledger that cannot record it
-- would have to grow a column later.

CREATE TABLE project_restarts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    project_id uuid NOT NULL REFERENCES projects(id),
    restarted_at TIMESTAMP(0) NOT NULL DEFAULT now()
);

-- The read is always "how many times has this reader restarted this project",
-- which is a count over exactly this pair.
CREATE INDEX project_restarts_user_project_index
    ON project_restarts (user_id, project_id);

CREATE TABLE task_attempts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    task_id uuid NOT NULL REFERENCES tasks(id),
    -- Which run through the project this belongs to: 1 until the reader
    -- restarts. Written by the api from `project_restarts`, not by the client,
    -- because a client that picks its own run number can rewrite history.
    run SMALLINT NOT NULL DEFAULT 1,
    outcome SMALLINT NOT NULL,
    points INTEGER NOT NULL DEFAULT 0,
    -- Whatever the CLI said about the failure. Capped at what the laravel
    -- request already capped it at, so the column cannot become a log sink.
    context VARCHAR(5000),
    created_at TIMESTAMP(0) NOT NULL DEFAULT now(),

    CONSTRAINT task_attempts_outcome_check CHECK (outcome IN (0, 1, 2, 3)),
    CONSTRAINT task_attempts_run_check CHECK (run >= 1)
);

COMMENT ON COLUMN task_attempts.outcome IS
    '0 attempted, 1 passed, 2 failed, 3 abandoned. Mapped by an enum in crates/api; change both together.';

-- Every read of this table is one reader's work on one task, newest first.
CREATE INDEX task_attempts_user_task_index
    ON task_attempts (user_id, task_id, created_at DESC);
