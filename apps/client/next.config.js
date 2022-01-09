const withTM = require("next-transpile-modules")(["three", "matrix"]);

module.exports = withTM({
  reactStrictMode: true,
});
