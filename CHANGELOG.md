# Changelog

What changed, newest first. The site's own `/changelog` page is written for
readers and lives in `web/app/data/Changelog.ts`; this file is for the repo.

## 2026-10-10

### Billing

- One Stripe customer per reader. The customer id is stored the moment it is
  minted, not when a checkout completes, so an abandoned checkout no longer
  leaves behind a customer nothing remembers. The row is locked while it is
  minted, so a sign-in and a checkout racing for one reader make one between
  them.
- Signing in or registering ensures the customer in the background. Sign-in
  never waits on Stripe and never fails because of it; a failure is a warning
  in the logs, carrying the user id and Stripe's error code only.
- Checkout ensures it again before redirecting, and refuses with
  `unavailable` rather than sending a reader to Stripe without one.
- A new Stripe customer carries the reader's name, so the dashboard shows
  who they are rather than whatever was typed at checkout. Customers that
  already exist keep the name they have.
- Customers minted twice before this change are still in Stripe.

## 2026-10-07

### Mobile

- A floating tab bar at the bottom of the screen replaces the header links
  below 720px: books, projects, pricing, blog, and join or the account sheet.
  It hides after 2.5s without input and comes back on any.
- The mobile header is the logo and search, nothing else.
- `join` in the tab bar opens the sign-in dialog, as it does on desktop.
- A compact three-column footer replaces the desktop footer below 720px.
- The segmented filter stays on one row across the full width.
- The newsletter input keeps its full height when the form stacks.
- The lesson breadcrumb no longer widens the page past the screen.

### Lessons

- A signed-in reader who paid no longer sees the paywall when their browser
  had cached the lesson from a signed-out visit. The unlock request skips the
  browser cache; `npm run test:unlock` in `web/` reproduces it in Chrome.

### Layout

- Less space under the header: 64px on desktop, 48px on phones. The projects
  page opens at the same gap as every other page.
- The faded end mark above the footer on book and project pages is gone.
- The viewport reaches the safe-area insets (`viewport-fit=cover`).
