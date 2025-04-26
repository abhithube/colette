import eslint from '@eslint/js'
import prettier from 'eslint-config-prettier/flat'
import react from 'eslint-plugin-react'
import reactHooks from 'eslint-plugin-react-hooks'
import globals from 'globals'
import tseslint from 'typescript-eslint'

export default tseslint.config(
  {
    files: ['**/*.{ts,tsx}'],
    ignores: ['**/dist/', '**/node_modules/'],
    languageOptions: {
      globals: globals.browser,
    },
  },
  eslint.configs.recommended,
  tseslint.configs.recommended,
  react.configs.flat.recommended,
  react.configs.flat['jsx-runtime'],
  {
    plugins: {
      'react-hooks': reactHooks,
    },
    rules: {
      'react-hooks/react-compiler': 'error',
    },
  },
  prettier,
)
