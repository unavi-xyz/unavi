import { env } from "../env/client.mjs";

export const getBaseUrl = () => {
  if (typeof window !== "undefined") return ""; // browser should use relative url
  if (env.VERCEL_URL) return `https://${env.VERCEL_URL}`; // SSR should use vercel url
  return "http://localhost:3000"; // dev SSR should use localhost
};
