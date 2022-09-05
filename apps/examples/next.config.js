const withTM = require("next-transpile-modules")(["@wired-labs/engine"]);

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
