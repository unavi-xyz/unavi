const withTM = require("next-transpile-modules")([
  "three",
  "3d",
  "ceramic",
  "scene",
]);

module.exports = withTM({
  reactStrictMode: true,
});
