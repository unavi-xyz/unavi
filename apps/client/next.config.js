const withTM = require("next-transpile-modules")(["three", "3d", "matrix"]);

module.exports = withTM({
  reactStrictMode: true,
});
