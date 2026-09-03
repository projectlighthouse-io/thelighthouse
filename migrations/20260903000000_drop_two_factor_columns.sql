-- users: drop the three two-factor columns.
--
-- They are Fortify's, inherited whole from the laravel baseline, and nothing
-- on this side ever grew into them. No query in `crates/` names any of the
-- three, and the settings page that appeared to own them was a stub: the
-- status read "not enabled" as literal markup and the Enable button carried
-- no handler. The rebuild has one sign-in path, and it is the provider's.
--
-- Not reversible. A secret is not recoverable once dropped, so a reader who
-- had enrolled under laravel would have to enrol again if this ever came
-- back — which is the trade being made by removing the feature, not a
-- side effect of the migration.

ALTER TABLE users DROP COLUMN two_factor_secret;
ALTER TABLE users DROP COLUMN two_factor_recovery_codes;
ALTER TABLE users DROP COLUMN two_factor_confirmed_at;
