const withTM = require("next-transpile-modules")(["three", "@wired-xr/new-engine"]);

module.exports = withTM({
  reactStrictMode: true,
});
