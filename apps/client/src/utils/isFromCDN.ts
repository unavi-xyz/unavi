import { env } from "../env.mjs";

export function isFromCDN(url?: string | null): boolean {
  if (!url || !env.NEXT_PUBLIC_CDN_ENDPOINT) return false;

  const host = new URL(url).host;
  const fromCDN = host.startsWith(env.NEXT_PUBLIC_CDN_ENDPOINT);

  return fromCDN;
}
