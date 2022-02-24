const withTM = require("next-transpile-modules")(["ceramic"]);

module.exports = withTM({
  reactStrictMode: true,
});
