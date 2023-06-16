import { env } from "../../../env.mjs";

const headers = env.NEXT_PUBLIC_CRYPTOAVATARS_API_KEY
  ? { "API-KEY": `$2b$10$${env.NEXT_PUBLIC_CRYPTOAVATARS_API_KEY}` }
  : undefined;

export const avatarFetcher = (url: string) =>
  fetch(url, { headers }).then((r) => r.json());
