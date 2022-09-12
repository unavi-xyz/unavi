const withPlugins = require("next-compose-plugins");
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
 * Don't be scared of the generics here.
 * All they do is to give us autocompletion when using this.
 *
 * @template {import('next').NextConfig} T
 * @param {T} config - A generic parameter that flows through to the return type
 * @constraint {{import('next').NextConfig}}
 */
function defineConfig(config) {
  return config;
}

const config = defineConfig({
  reactStrictMode: true,
  swcMinify: true,
  experimental: {
    images: {
      allowFutureImage: true,
    },
  },
  images: {
    domains: [
      `${process.env.S3_BUCKET}.${process.env.S3_ENDPOINT}`,
      process.env.NEXT_PUBLIC_IPFS_GATEWAY,
      "avatar.tobi.sh",
    ],
  },
  async redirects() {
    return [
      {
        source: "/app",
        destination: "/",
        permanent: false,
      },
      {
        source: "/editor",
        destination: "/create",
        permanent: false,
      },
    ];
  },
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
  webpack: function (config) {
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
      syncWebAssembly: true,
    };
    return config;
  },
});

module.exports = withPlugins(
  [withBundleAnalyzer, withAxiom, withTM, withPWA],
  config
);
