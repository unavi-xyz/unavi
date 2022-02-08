const withTM = require("next-transpile-modules")([
  "three",
  "3d",
  "avatars",
  "ceramic",
  "ui",
]);

module.exports = withTM({
  reactStrictMode: true,
});
