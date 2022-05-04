const withTM = require("next-transpile-modules")([
  "three",
  "3d",
  "ceramic",
  "scene",
]);

module.exports = withTM({
  reactStrictMode: true,
  async redirects() {
    return [
      {
        source: "/studio",
        destination: "/create",
        permanent: true,
      },
    ];
  },
});
