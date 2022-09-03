const withTM = require("next-transpile-modules")(["@wired-xr/engine"]);

module.exports = withTM({
  reactStrictMode: true,
  webpack: function (config) {
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
      syncWebAssembly: true,
      topLevelAwait: true,
    };
    return config;
  },
});
