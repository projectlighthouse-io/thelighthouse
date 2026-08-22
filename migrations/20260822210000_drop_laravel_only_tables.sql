-- Three more tables the rebuild has no use for.
--
--   notifications          laravel's database notification channel. The rebuild
--                          has no notification system, and if it grows one it
--                          will not be this shape — the table is a polymorphic
--                          `notifiable_type`/`notifiable_id` pair with a JSON
--                          payload, which is laravel's convention, not a design
--                          the rust side would arrive at.
--   password_reset_tokens  there is no password to reset. Sign-in is GitHub and
--                          Google only, and the api has no local credential to
--                          issue a reset for.
--   votes                  roadmap voting. Nothing in the rebuild reads it yet;
--                          dropping it now means the feature comes back with a
--                          schema chosen for it rather than inherited.
--
-- Nothing references any of the three. `votes` is the only one with a foreign
-- key of its own — to users — and that goes with the table. `users` is not
-- touched.
--
-- All three are empty in development. `notifications` and
-- `password_reset_tokens` are transient by nature and losing them costs
-- nothing; `votes` is not. If readers have voted on the roadmap in production,
-- that is a record of what they asked for, and it is gone after this. Worth a
-- look before the cutover — the counts are the whole value, and they are cheap
-- to export.

DROP TABLE notifications;
DROP TABLE password_reset_tokens;
DROP TABLE votes;
