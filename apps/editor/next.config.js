const withTM = require("next-transpile-modules")([
  "three",
  "3d",
  "ui",
  "ceramic",
]);

module.exports = withTM({
  reactStrictMode: true,
});
