# Changelog

What changed, newest first. The site's own `/changelog` page is written for
readers and lives in `web/app/data/Changelog.ts`; this file is for the repo.

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
