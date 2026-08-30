-- task_validators: the table is gone, because nothing on this side validates.
--
-- A validator is a probe and an expectation — "connect to :4221, expect a 200"
-- — and it is luxctl that runs it, on the reader's own machine, against the
-- program they just wrote. The server never sees the program, so it could not
-- check it if it wanted to.
--
-- The rows were never the source anyway: `validator_dsl` was synced out of the
-- .bp file, and the CLI reads its copy of the blueprint. The laravel api
-- confirms it — `CliTaskData` shipped `validators: []`, hardcoded, on every
-- response. This drops a table whose only reader was the sync that filled it.

DROP TABLE task_validators;
