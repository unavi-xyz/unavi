const withTM = require("next-transpile-modules")([
  "three",
  "@wired-labs/engine",
  "@wired-labs/lens",
]);

/**
 * @type {import('next').NextConfig}
 */
module.exports = withTM({
  eslint: {
    ignoreDuringBuilds: true,
  },
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
