const withTM = require("next-transpile-modules")(["three", "3d", "ceramic"]);

module.exports = withTM({
  reactStrictMode: true,
});
