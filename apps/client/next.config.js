const { withAxiom } = require("next-axiom");
const withTM = require("next-transpile-modules")([
  "three",
  "@wired-labs/engine",
  "@wired-labs/lens",
]);
const runtimeCaching = require("next-pwa/cache.js");
const withPWA = require("next-pwa")({
  dest: "public",
  disable: process.env.NODE_ENV === "development",
  runtimeCaching,
});
const withBundleAnalyzer = require("@next/bundle-analyzer")({
  enabled:
    process.env.BUNDLE_ANALYZE === "true" &&
    process.env.NODE_ENV === "production",
});

/**
 * @type {import('next').NextConfig}
 */
const config = {
  i18n: {
    locales: ["en"],
    defaultLocale: "en",
  },
  images: {
    domains: [
      `${process.env.S3_BUCKET}.${process.env.S3_ENDPOINT}`,
      `${process.env.NEXT_PUBLIC_IPFS_GATEWAY}`,
      "avatar.tobi.sh",
    ],
  },
  reactStrictMode: true,
  swcMinify: true,
  async headers() {
    return [
      {
        source: "/:path*",
        headers: [
          {
            key: "Cross-Origin-Opener-Policy",
            value: "same-origin",
          },
          {
            key: "Cross-Origin-Embedder-Policy",
            value: "require-corp",
          },
        ],
      },
    ];
  },
  async redirects() {
    return [
      {
        source: "/app",
        destination: "/explore",
        permanent: false,
      },
      {
        source: "/editor",
        destination: "/create",
        permanent: false,
      },
    ];
  },
  webpack: function (config) {
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
      syncWebAssembly: true,
    };
    return config;
  },
};

module.exports = (_phase, { defaultConfig }) => {
  // Workaround to avoid console warning spam
  // https://github.com/vercel/next.js/issues/39161
  delete defaultConfig.webpackDevMiddleware;
  delete defaultConfig.configOrigin;
  delete defaultConfig.target;
  delete defaultConfig.webpack5;
  delete defaultConfig.amp.canonicalBase;
  delete defaultConfig.experimental.outputFileTracingRoot;

  const plugins = [withBundleAnalyzer, withAxiom, withTM, withPWA];
  return plugins.reduce((acc, plugin) => plugin(acc), {
    ...defaultConfig,
    ...config,
  });
};
