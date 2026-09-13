-- A membership that never runs out, and an end date that is checked.
--
-- Two halves of one gap. `grants_access` was `status = active` and nothing
-- else, so a row whose `period_ends_at` had passed went on granting the track
-- for as long as the status said active. In practice a provider webhook flips
-- the status and the two agree — but a webhook that never arrives, or arrives
-- and fails, leaves a reader reading forever, and the row says plainly that it
-- should not. Nothing above this line could tell, because nothing above this
-- line looked at the date.
--
-- Making the date decide needs a way to say "there is no date", because there
-- is a real case: a track bought outright. `period_ends_at` is already
-- nullable, but null there has meant "we were not told when the period ends"
-- since the table was written — a manual scholarship row has one. Reading null
-- as "forever" would turn every one of those into lifetime access, silently,
-- on the deploy that started checking.
--
-- So the flag says it instead. `lifetime = true` is the deliberate statement
-- that this membership has no end and none is expected; null `period_ends_at`
-- on its own stays what it was — unknown — and grants nothing once the date is
-- what decides. The two are different facts and neither can stand in for the
-- other.
--
-- Default false, so every row already here keeps meaning exactly what it meant.

ALTER TABLE memberships
    ADD COLUMN lifetime BOOLEAN NOT NULL DEFAULT FALSE;

-- A lifetime membership has no end, and a dated one is not lifetime. Enforced
-- here rather than trusted to the writer: the two columns answer one question
-- between them, and a row that sets both is a row nobody can act on.
ALTER TABLE memberships
    ADD CONSTRAINT memberships_lifetime_has_no_end
    CHECK (NOT (lifetime AND period_ends_at IS NOT NULL));
