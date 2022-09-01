import { withAxiom } from "next-axiom";
import withPWA from "next-pwa";
import runtimeCaching from "next-pwa/cache.js";
import createTM from "next-transpile-modules";

const withTM = createTM([
  "three",
  "@wired-xr/engine",
  "@wired-xr/ipfs",
  "@wired-xr/lens",
]);

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

const config = withAxiom(
  withTM(
    defineConfig({
      reactStrictMode: true,
      swcMinify: true,
      experimental: {
        images: {
          allowFutureImage: true,
        },
      },
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
          {
            source: "/studio",
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
      pwa: {
        dest: "public",
        runtimeCaching,
      },
      webpack: function (config) {
        config.experiments = {
          ...config.experiments,
          asyncWebAssembly: true,
          syncWebAssembly: true,
        };
        return config;
      },
    })
  )
);

export default process.env.NODE_ENV === "development"
  ? config
  : withPWA(config);
