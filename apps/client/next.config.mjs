import { withAxiom } from "next-axiom";

import { env } from "./src/env.mjs";

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
    value: "credentialless",
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
async function defineNextConfig(config) {
  const plugins = [];

  if (env.NEXT_PUBLIC_AXIOM_INGEST_ENDPOINT) plugins.push(withAxiom);

  if (env.NODE_ENV === "production" && env.DISABLE_PWA !== "true") {
    const withPWAInit = (await import("@ducanh2912/next-pwa")).default;
    const withPWA = withPWAInit({ dest: "public" });
    plugins.push(withPWA);
  }

  return plugins.reduce((acc, plugin) => plugin(acc), config);
}

export default await defineNextConfig({
  crossOrigin: "anonymous",
  eslint: {
    ignoreDuringBuilds: true,
  },
  experimental: {
    appDir: true,
    outputFileTracingExcludes: { "**": ["**swc/core**"] },
  },
  async headers() {
    return [
      {
        headers: [...securityHeaders, ...isolationHeaders],
        source: "/:path*",
      },
    ];
  },
  images: {
    domains: [env.NEXT_PUBLIC_CDN_ENDPOINT],
  },
  outputFileTracing: true,
  productionBrowserSourceMaps: true,
  reactStrictMode: true,
  transpilePackages: ["engine", "contracts"],
  webpack: function (config) {
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
    };
    return config;
  },
});
