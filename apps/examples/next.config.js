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
  i18n: {
    locales: ["en"],
    defaultLocale: "en",
  },
  reactStrictMode: true,
  swcMinify: true,
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
