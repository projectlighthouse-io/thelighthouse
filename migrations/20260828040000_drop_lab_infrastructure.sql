-- The lab tables. Virtual labs are off, so these describe machinery that no
-- longer exists.
--
-- `docs/rebuild.md` retires the whole of it: no sentinel, no forge, no VM
-- lifecycle, no SSH keys, no terminals, and DSA code execution with them. What
-- survives is luxctl's build-your-own projects, which are `projects` and
-- `tasks` and never needed a lab row.
--
-- The three tables are a closed cluster. `lab_submissions` points at `labs` and
-- at `users`; `user_lab_ssh_keys` points at `users`; nothing anywhere points at
-- any of them. So this is three drops and a column, with no constraint to
-- unpick first beyond the child-before-parent order below.
--
--   lab_submissions    one row per code run: language, outcome, test counts,
--                      the code, the raw output.
--   labs               the VM spec — image, memory, cpus, tier. Derived from
--                      the content repo, so it was always regenerable.
--   user_lab_ssh_keys  a keypair per reader, for SSH into a container that no
--                      longer gets started.
--
-- **`lab_submissions` is reader history and is the one thing here worth
-- keeping a copy of.** `labs` regenerates from yaml and the keypairs are
-- useless without VMs, but a submission is a record of somebody's work — what
-- they wrote, whether it passed, how long it took. It is empty in development.
-- If production has rows, export them before the cutover; after this migration
-- the question "how many people finished the DSA labs" has no answer.
--
-- **`user_lab_ssh_keys.private_key` is a `text` column holding private keys in
-- the clear.** That is reason to drop it rather than reason to hesitate: the
-- rows unlock nothing now, and until this runs, production is storing readers'
-- private keys in plaintext for a feature that no longer exists. Nothing needs
-- exporting here — a keypair for a machine that will never boot is not a
-- record of anything.

-- Child first: lab_submissions_lab_id_foreign would refuse the drop otherwise,
-- and a CASCADE would take whatever else happened to reference labs without
-- naming it.
DROP TABLE lab_submissions;
DROP TABLE labs;

DROP TABLE user_lab_ssh_keys;

-- projects.lab_slug
--
-- How a project said "run me on this lab". A plain varchar with no foreign key,
-- so nothing failed when `labs` went — it just became a string naming a table
-- that is not there. `projects` itself stays: luxctl still fetches projects and
-- their tasks, and a project that needs no VM is the only kind left.
ALTER TABLE projects DROP COLUMN lab_slug;
