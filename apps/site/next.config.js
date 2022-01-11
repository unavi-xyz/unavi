const withTM = require("next-transpile-modules")(["ui", "matrix"]);

module.exports = withTM({
  reactStrictMode: true,
});
