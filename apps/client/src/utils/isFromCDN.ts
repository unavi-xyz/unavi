import { env } from "../env/client.mjs";

export function isFromCDN(url?: string | null): boolean {
  if (!url || !env.NEXT_PUBLIC_CDN_ENDPOINT) return false;

  const host = url?.replace(/^https?:\/\//, "");
  const isFromCDN = host?.startsWith(env.NEXT_PUBLIC_CDN_ENDPOINT);

  return isFromCDN;
}
