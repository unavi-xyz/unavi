const withPWA = require("next-pwa");
const runtimeCaching = require("next-pwa/cache");
const withTM = require("next-transpile-modules")([
  "three",
  "@wired-xr/engine",
  "@wired-xr/new-engine",
  "@wired-xr/ethers",
  "@wired-xr/ipfs",
  "@wired-xr/lens",
]);

module.exports = withPWA(
  withTM({
    reactStrictMode: true,
    images: {
      domains: ["ipfs.infura.io", "avatar.tobi.sh"],
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
    pwa: {
      dest: "public",
      disable: process.env.NODE_ENV === "development",
      runtimeCaching,
    },
  })
);
