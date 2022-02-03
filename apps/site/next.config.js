const withTM = require("next-transpile-modules")(["ui", "ceramic"]);

module.exports = withTM({
  reactStrictMode: true,
});
