import createPWA from "@ducanh2912/next-pwa";
import { withAxiom } from "next-axiom";

import { env } from "./src/env/server.mjs";

const withPWA = createPWA({
  dest: "public",
  disable: env.NODE_ENV === "development",
});

const securityHeaders = [
  {
    key: "X-DNS-Prefetch-Control",
    value: "on",
  },
  {
    key: "Strict-Transport-Security",
    value: "max-age=63072000; includeSubDomains; preload",
  },
  {
    key: "X-XSS-Protection",
    value: "1; mode=block",
  },
  {
    key: "X-Frame-Options",
    value: "SAMEORIGIN",
  },
  {
    key: "X-Content-Type-Options",
    value: "nosniff",
  },
  {
    key: "Referrer-Policy",
    value: "strict-origin-when-cross-origin",
  },
  {
    key: "Permissions-Policy",
    value: "camera=(), microphone=(self), geolocation=()",
  },
];

const isolationHeaders = [
  {
    key: "Cross-Origin-Opener-Policy",
    value: "same-origin",
  },
  {
    key: "Cross-Origin-Embedder-Policy",
    value: "require-corp",
  },
];

/**
 * Don't be scared of the generics here.
 * All they do is to give us autocompletion when using this.
 *
 * @template {import('next').NextConfig} T
 * @param {T} config - A generic parameter that flows through to the return type
 * @constraint {{import('next').NextConfig}}
 */
function defineNextConfig(config) {
  const plugins = [withAxiom, withPWA];
  return plugins.reduce((acc, plugin) => plugin(acc), config);
}

export default defineNextConfig({
  crossOrigin: "anonymous",
  eslint: {
    ignoreDuringBuilds: true,
  },
  experimental: {
    appDir: true,
  },
  images: {
    domains: [env.NEXT_PUBLIC_CDN_ENDPOINT],
  },
  output: "standalone",
  reactStrictMode: true,
  transpilePackages: ["engine", "contracts", "protocol"],
  async headers() {
    return [
      {
        source: "/:path*",
        headers: [...securityHeaders, ...isolationHeaders],
      },
    ];
  },
  async redirects() {
    return [
      {
        source: "/play",
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
    };
    return config;
  },
});
