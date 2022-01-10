const withTM = require("next-transpile-modules")(["matrix"]);

module.exports = withTM({
  reactStrictMode: true,
});
