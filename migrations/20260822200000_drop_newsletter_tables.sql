-- The newsletter tables go: newsletter_opens, newsletters, newsletter_subscribers.
--
--   newsletters             issues written and sent from the app
--   newsletter_subscribers  the list, one row per email
--   newsletter_opens        one row per open, from a tracking pixel
--
-- The rebuild has no newsletter feature, so none of these has a writer or a
-- reader. Dropped in dependency order rather than with CASCADE: only
-- newsletter_opens has foreign keys — to newsletters and to users — so removing
-- it first lets the other two go on their own. CASCADE would work too and would
-- also quietly remove anything else pointing at them, which is exactly the
-- report you want to read rather than not receive.
--
-- `users` is not touched. It is only ever the target of the opens foreign key,
-- and dropping the referencing table takes the constraint with it.
--
-- **This destroys data that cannot be recomputed.** Everything dropped so far
-- was a column the rebuild replaced; these are records. All three are empty in
-- development, but production has been sending newsletters, and the subscriber
-- list in particular is the kind of thing whose absence is noticed much later.
-- Two things worth settling before this runs anywhere real:
--
--   * the platform sends through MailerLite and ConvertKit, so the authoritative
--     list is very likely already there and these rows are a local mirror —
--     worth confirming rather than assuming;
--   * if it is not, export before the cutover. There is no recovering it after.

DROP TABLE newsletter_opens;
DROP TABLE newsletters;
DROP TABLE newsletter_subscribers;
