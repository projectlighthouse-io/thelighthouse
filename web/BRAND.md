# Brand

The mark is a lighthouse lamp in an open frame. Every file below is used as
given — never redrawn, re-exported or optimised.

## Where the files live

| file | for |
|------|-----|
| `public/favicon.ico` (16, 32, 48), `favicon.svg`, `favicon-16/32/48.png` | browser tabs |
| `public/apple-touch-icon.png` | iOS home screen — 180×180, full-bleed dark |
| `public/icon-192.png`, `icon-512.png`, `icon-maskable-512.png` | the web manifest |
| `public/safari-pinned-tab.svg` | Safari pinned tabs — monochrome mask |
| `public/site.webmanifest` | the PWA manifest |
| `public/og-image.png` | the share card, 1200×630 |
| `public/brand/app-icon.svg` | the mark on the dark tile — the navbar logo |
| `public/brand/logo.svg` | bare mark, #202020, for light grounds |
| `public/brand/logo-dark.svg` | bare mark, #f2f2f2, for dark grounds |
| `public/brand/lockup.svg`, `lockup-dark.svg` | tile + wordmark, for emails, docs, READMEs |
| `public/brand/logo-512.png`, `logo-dark-512.png` | raster, for places that cannot take SVG |
| `public/lighthouse.svg` | the app tile (same file as `brand/app-icon.svg`), kept at the old URL for links from outside the site |

On the site, the mark only ever comes from `<UiLogo>` (`app/components/Ui/Logo.vue`),
which inlines those drawings:

```vue
<UiLogo />                         <!-- tile, 24px: the default -->
<UiLogo variant="mark" :size="20" />
<UiLogo variant="mark-dark" />
<UiLogo :size="32" label="Project Lighthouse" />  <!-- standalone: role="img" -->
```

Next to text, it is decorative and the link around it carries the name
(`aria-label="Project Lighthouse home"`). Standing alone, give it a `label`.

## Rules

**Colours.** Ink `#202020`, light `#f2f2f2`, lamp `#f2a41f`. The lamp is always
yellow — the one exception is the monochrome `safari-pinned-tab.svg`.

**Grid.** The mark is drawn on a 32px grid. The tile's corner radius is 5/32 of
its size.

**Minimum sizes.** 16px for the tile, 20px for the bare mark.

**Clear space.** At least ¼ of the logo's size on every side.

**On dark grounds** use the tile as it is, or `mark-dark`. Never the ink mark on
a dark ground.

**Don't**

- recolour the lamp
- add effects, shadows or outlines
- stretch it
- rotate it
- use a CSS `filter` on it — `invert()` included
- set the wordmark in any font other than Geist Mono
