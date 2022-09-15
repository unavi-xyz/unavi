module.exports = {
  plugins: ["tailwindcss", "simple-import-sort", "json", "json-files"],
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
    "tailwindcss/no-custom-classname": "off",
    "simple-import-sort/imports": "error",
    "simple-import-sort/exports": "error",
    "json-files/require-unique-dependency-names": "error",
    "json-files/sort-package-json": "error",
  },
};
