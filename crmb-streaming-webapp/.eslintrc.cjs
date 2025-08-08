module.exports = {
  root: true,
  env: { browser: true, es2020: true },
  extends: [
    'eslint:recommended',
    'plugin:react/recommended',
    'plugin:react-hooks/recommended'
  ],
  ignorePatterns: ['dist', '.eslintrc.cjs'],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true
    }
  },
  plugins: ['react-refresh', 'react', 'react-hooks'],
  settings: {
    react: {
      version: 'detect',
      runtime: 'automatic'
    }
  },
  rules: {
    'react-refresh/only-export-components': 'warn',
    'react/react-in-jsx-scope': 'off',
    'prefer-const': 'error',
    'no-var': 'error',
    'no-console': 'warn',
    'no-debugger': 'error',
    'no-unused-vars': 'off',
    'no-undef': 'off',
    'eqeqeq': ['error', 'always'],
    'curly': ['error', 'all']
  },
}