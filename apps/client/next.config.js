const withTM = require("next-transpile-modules")(["three", "3d"]);

module.exports = withTM({
  reactStrictMode: true,
});
