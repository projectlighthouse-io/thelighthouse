// @ts-check
import withNuxt from './.nuxt/eslint.config.mjs'

export default withNuxt(
  {
    // k6 runs bench/load.js in its own runtime, which injects __ENV and open().
    // Linting it against browser/node globals reports phantom errors.
    files: ['bench/**'],
    languageOptions: {
      globals: { __ENV: 'readonly', __VU: 'readonly', __ITER: 'readonly', open: 'readonly' },
    },
  },
)
