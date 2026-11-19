# Changelog

What changed, newest first. The site's own `/changelog` page is written for
readers and lives in `web/app/data/Changelog.ts`; this file is for the repo.

## 2026-11-19

### Billing

- A reader still renewing a `voyage_*` plan reads the Foundations books
  again. Those plans were taken out of `pricing.yaml`, and a plan the api
  could not find covered nothing, so they saw the paywall. Laravel's plain
  `voyage` plan still opens everything.

## 2026-11-17

### Security

- An article whose markdown asks for runaway emphasis matching (thousands of
  unmatched `*`, `_` or `~`) is shown as plain text instead of rendered. One
  such article could hold the Nuxt process for tens of seconds per view.
- Google sign-in uses the email address only when Google says it is verified.
- Signing in never moves an account onto a new GitHub or Google id: an email
  that already belongs to an account linked to a different id is refused.
- The origin answers only requests that came through Cloudflare. Caddy
  refuses anything without the `X-Origin-Secret` header a Cloudflare
  transform rule adds, so the App Platform hostname no longer skips the
  edge. The image will not boot without `ORIGIN_SECRET`.
- Rate limiting moves to Cloudflare. The api's own limit is now a backstop
  that counts a reader or a Cloudflare address only; nitro's server-side
  renders no longer share one budget any visitor could spend.

### Blog

- An author can publish one new article every ten minutes. Inside that
  window the api answers 429 with `Retry-After`; edits are not limited.

### Operations

- The signed `POST /reload` endpoint and `make content-reload` are gone.
  Content reaches production with each image; a running container still
  rereads it on SIGHUP (`make content-sync`).

### Billing

- A one-time purchase grants its books only once the payment has settled. A
  delayed method such as a bank debit grants on
  `checkout.session.async_payment_succeeded`, not when checkout finishes.

## 2026-11-16

### Reader

- Bangla lessons set Bengali text in Hind Siliguri instead of Noto Serif
  Bengali, self-hosted and scaled 112% to sit level with Literata. The font
  only downloads on pages that contain Bengali.

## 2026-11-15

### Reader

- `<term tip="…">` renders again: a wavy underline that shows its definition
  on hover, focus or tap. The tip is markdown, rendered and escaped server
  side; every other raw HTML in a lesson is still omitted.
- `> [!NOTE]` and the other GitHub alerts render as callouts instead of a
  quote with the marker printed in it.
- Lesson prose is set in Literata. English paragraphs are justified and
  hyphenated; inline code and links never are.
- A translated lesson says so in a yellow notice at the top, linking the
  other language with its flag. Bangla pages load Noto Serif Bengali, and
  quotes and emphasis there stay upright rather than taking a faux slant.

## 2026-11-12

### luxctl

- The project and task pages showed `luxctl projects start` and
  `luxctl tasks submit`, neither of which exists. They now show
  `luxctl project start --id`, then `luxctl validate` on a project and
  `luxctl run --task` on a task.
- The api's 401 told readers to run `lux login`, which is not a command. It
  now names `luxctl auth --token <TOKEN>`.

## 2026-10-12

### Reader

- A heading with inline html — the `<span>`-coloured "Mutual Exclusion" in Go
  Intermediate's sync primitives — showed its tags in the contents list, and
  its entry linked to an id the page never had. Contents entries now take the
  heading's text the way comrak reads it, so they show the words and scroll to
  the heading.

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
