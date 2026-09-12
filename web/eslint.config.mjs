// @ts-check
import withNuxt from './.nuxt/eslint.config.mjs'

export default withNuxt(
  {
    // `app/data/Catalogue.ts` is written by `lighthouse-prices catalogue` out
    // of pricing.yaml and ohara. It is json inside a `.ts` file — quoted keys,
    // double quotes, no trailing commas — and reformatting it to satisfy style
    // rules would be undone by the next regeneration.
    ignores: ['app/data/Catalogue.ts'],
  },
  {
    // k6 runs bench/load.js in its own runtime, which injects __ENV and open().
    // Linting it against browser/node globals reports phantom errors.
    files: ['bench/**'],
    languageOptions: {
      globals: { __ENV: 'readonly', __VU: 'readonly', __ITER: 'readonly', open: 'readonly' },
    },
  },
)
