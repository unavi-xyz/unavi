import { z } from "zod";

import { env } from "../env.mjs";

const HOST = new URL(env.NEXT_PUBLIC_DEPLOYED_URL).host;

// If host includes a port, replace : with %3A
const HOST_ENCODED = HOST.replace(":", "%3A");

export function createDidWeb(id: string) {
  return `did:web:${HOST_ENCODED}:api:v1:dids:${id}`;
}

export const CustomDIDDocumentSchema = z.object({
  profile: z
    .object({
      address: z.string(),
      background: z.string(),
      bio: z.string(),
      image: z.string(),
      username: z.string(),
    })
    .partial(),
});

export type CustomDIDDocument = z.infer<typeof CustomDIDDocumentSchema>;
