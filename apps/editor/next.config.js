const withTM = require("next-transpile-modules")([
  "three",
  "3d",
  "ui",
  "matrix",
]);

module.exports = withTM({
  reactStrictMode: true,
});
