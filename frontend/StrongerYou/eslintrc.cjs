module.exports = {
    root: true,
    parser: '@typescript-eslint/parser',  // Use the TypeScript parser
    plugins: [
      'eslint-plugin-react',
      '@typescript-eslint',  // Include the TypeScript plugin
      'react-hooks'
    ],
    extends: [
      'eslint:recommended',  // Recommended ESLint rules
      'plugin:@typescript-eslint/recommended',  // Recommended TypeScript rules
      'plugin:react/recommended',
      'plugin:react-hooks/recommended'
    ],
    env: {
      browser: true,  //  for browser code
      es2020: true,
      node: true
    },
    settings: {
      react: {
        version: 'detect',  // Automatically detect React version
      },
    },
    rules: {
      //  customize rules here
      'no-console': 'off',  // Allow console.log()
    },
    ignorePatterns: [
      "dist",
      "node_modules"
    ]
  };