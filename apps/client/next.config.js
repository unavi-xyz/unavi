const withTM = require("next-transpile-modules")([
  "three",
  "@wired-xr/scene",
  "@wired-xr/avatar",
]);

module.exports = withTM({
  reactStrictMode: true,
  async redirects() {
    return [
      {
        source: "/app",
        destination: "/",
        permanent: false,
      },
    ];
  },
});
