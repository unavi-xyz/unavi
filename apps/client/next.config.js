const withTM = require("next-transpile-modules")(["three", "@wired-xr/scene"]);

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
