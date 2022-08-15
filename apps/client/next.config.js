const withPWA = require("next-pwa");
const runtimeCaching = require("next-pwa/cache");
const withTM = require("next-transpile-modules")([
  "@wired-xr/engine",
  "@wired-xr/ipfs",
  "@wired-xr/lens",
]);

const settings = withTM({
  reactStrictMode: true,
  images: {
    domains: [process.env.NEXT_PUBLIC_IPFS_GATEWAY, "avatar.tobi.sh"],
  },
  async redirects() {
    return [
      {
        source: "/app",
        destination: "/",
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
  pwa: {
    dest: "public",
    runtimeCaching,
  },
  webpack: function (config) {
    config.experiments.syncWebAssembly = true;
    config.experiments.asyncWebAssembly = true;
    return config;
  },
});

module.exports = process.env.NODE_ENV === "development" ? settings : withPWA(settings);
