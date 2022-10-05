import withRawBundleAnalyzer from "@next/bundle-analyzer";
import { withAxiom } from "next-axiom";
import withRawPWA from "next-pwa";
import runtimeCaching from "next-pwa/cache.js";
import withRawTM from "next-transpile-modules";

import { env } from "./src/env/server.mjs";

const withBundleAnalyzer = withRawBundleAnalyzer({
  enabled: env.BUNDLE_ANALYZE === "true" && env.NODE_ENV === "production",
});

const withPWA = withRawPWA({
  dest: "public",
  disable: env.NODE_ENV === "development",
  runtimeCaching,
});

const withTM = withRawTM(["three", "@wired-labs/engine", "@wired-labs/lens"]);

/**
 * Don't be scared of the generics here.
 * All they do is to give us autocompletion when using this.
 *
 * @template {import('next').NextConfig} T
 * @param {T} config - A generic parameter that flows through to the return type
 * @constraint {{import('next').NextConfig}}
 */
function defineNextConfig(config) {
  const plugins = [withBundleAnalyzer, withAxiom, withTM, withPWA];
  return plugins.reduce((acc, plugin) => plugin(acc), {
    ...config,
  });
}

export default defineNextConfig({
  eslint: {
    ignoreDuringBuilds: true,
  },
  i18n: {
    locales: ["en"],
    defaultLocale: "en",
  },
  images: {
    domains: [
      `${env.S3_BUCKET}.${env.S3_ENDPOINT}`,
      `${env.NEXT_PUBLIC_IPFS_GATEWAY}`,
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
});
