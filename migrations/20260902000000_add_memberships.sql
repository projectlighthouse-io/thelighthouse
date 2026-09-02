-- memberships: one row per subscription a reader has, or used to have.
--
-- A new table rather than a use of `subscriptions`, which migration 0 carries
-- across from laravel. That one is Cashier's shape — `stripe_id`,
-- `stripe_status`, `stripe_price` — and it is still being written by the php
-- app, which serves readers until the cutover. Two writers on one table during
-- a rebuild is the kind of overlap that is only discovered from a support
-- ticket. The laravel table is left alone; carrying its rows across is a
-- backfill, deliberately separate from this.
--
-- The column names are also the reason for a new table rather than a rename.
-- The provider sits behind a trait here, and a schema that spells `stripe_` on
-- three of its columns has already decided the answer to a question the code
-- above it goes out of its way to leave open.
--
-- `plan` is our name for what was bought — `voyage_yearly` — never the
-- provider's price handle. Handles are immutable at stripe and get retired;
-- retiring one must not make an existing membership unreadable. The handle
-- lives in the gitignored plan config and nowhere else.
--
-- `status` is the four states worth telling apart, not the provider's eight:
--
--   0 active   paid up, and the only value that grants access
--   1 grace    a payment failed and the provider is retrying
--   2 ended    over, whether it ran out or somebody ended it
--
-- `past_due` locks immediately, which is what the laravel app did — only
-- `active` and `trialing` ever counted there. Keeping access through a dunning
-- cycle is a decision somebody can make later, and it should be made on
-- purpose rather than inherited from a match arm.
--
-- `provider` and `provider_ref` are what the row can be found by when a
-- webhook arrives knowing only the provider's own id.
--
-- `provider_ref` is nullable because a membership does not have to come from a
-- provider. A scholarship is `provider = 'manual'` and no reference: nothing
-- was charged, so there is nothing to name. That case is deliberately not the
-- billing crate's concern — it is a row, not a payment.
--
-- No ON DELETE on the user key, matching entitlements. Cascading would make
-- deleting a user silently destroy the record of what they paid for, and that
-- record is what an argument about a refund is settled with.

CREATE TABLE memberships (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    plan VARCHAR(64) NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,
    provider VARCHAR(32) NOT NULL,
    provider_ref VARCHAR(255),
    started_at TIMESTAMP(0) NOT NULL DEFAULT now(),
    period_ends_at TIMESTAMP(0),
    cancel_at TIMESTAMP(0),
    ended_at TIMESTAMP(0),

    CONSTRAINT memberships_status_check CHECK (status = ANY (ARRAY[0, 1, 2]))
);

-- What every webhook looks the row up by, and what makes a redelivered event
-- an upsert rather than a second membership. Stripe retries a delivery it does
-- not get a 2xx for, so the same event arriving twice is normal traffic.
--
-- Partial, because the manual rows all have a null reference and several nulls
-- are not a conflict in postgres anyway — stating it keeps the intent legible.
CREATE UNIQUE INDEX memberships_provider_ref_unique
    ON memberships (provider, provider_ref)
    WHERE provider_ref IS NOT NULL;

-- One live membership per reader. Two would make "is this reader subscribed"
-- a question with two answers, and the entitlement check would have to pick.
--
-- Ended rows are excluded so the history survives: resubscribing after
-- cancelling is a new row, and the old one stays as the record that it
-- happened.
CREATE UNIQUE INDEX memberships_one_live_per_user
    ON memberships (user_id)
    WHERE status <> 2;

-- The read path is "this reader's membership", which the partial unique index
-- above cannot serve once a reader has ended rows too.
CREATE INDEX memberships_user_id_index ON memberships (user_id);
