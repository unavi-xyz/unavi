const withTM = require("next-transpile-modules")(["3d", "matrix"]);

module.exports = withTM({
  reactStrictMode: true,
});
