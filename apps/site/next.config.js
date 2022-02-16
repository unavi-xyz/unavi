const withTM = require("next-transpile-modules")([
  "three",
  "3d",
  "avatars",
  "ceramic",
]);

module.exports = withTM({
  reactStrictMode: true,
});
