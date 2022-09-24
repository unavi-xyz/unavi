module.exports = {
  plugins: [
    "tailwindcss",
    "simple-import-sort",
    "json",
    "json-files",
    "unused-imports",
  ],
  extends: [
    "next/core-web-vitals",
    "turbo",
    "prettier",
    "plugin:tailwindcss/recommended",
    "plugin:json/recommended",
  ],
  rules: {
    "@next/next/no-html-link-for-pages": "off",
    "no-console": ["warn", { allow: ["info", "warn", "error"] }],
    "simple-import-sort/imports": "warn",
    "simple-import-sort/exports": "warn",
    "unused-imports/no-unused-imports": "error",
    "json-files/require-unique-dependency-names": "error",
    "json-files/sort-package-json": "warn",
  },
};
