const withTM = require("next-transpile-modules")(["three", "@wired-xr/engine"]);

module.exports = withTM({
  reactStrictMode: true,
});
