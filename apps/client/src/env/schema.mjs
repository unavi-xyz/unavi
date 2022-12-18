// @ts-check
import { z } from "zod";

/*
 * Specify your server-side environment variables schema here.
 * This way you can ensure the app isn't built with invalid env vars.
 */
export const serverSchema = z.object({
  BUNDLE_ANALYZE: z.string().optional(),
  DATABASE_URL: z.string().url(),
  ETH_PROVIDER: z.string().url(),
  NEXTAUTH_SECRET: z.string(),
  NEXTAUTH_URL: z.string().url().optional(),
  NODE_ENV: z.enum(["development", "test", "production"]),
  S3_ACCESS_KEY_ID: z.string(),
  S3_BUCKET: z.string(),
  S3_ENDPOINT: z.string(),
  S3_REGION: z.string(),
  S3_SECRET: z.string(),
});

/*
 * Specify your client-side environment variables schema here.
 * This way you can ensure the app isn't built with invalid env vars.
 * To expose them to the client, prefix them with `NEXT_PUBLIC_`.
 */
export const clientSchema = z.object({
  NEXT_PUBLIC_ALCHEMY_ID: z.string().optional(),
  NEXT_PUBLIC_AXIOM_INGEST_ENDPOINT: z.string().optional(),
  NEXT_PUBLIC_CDN_ENDPOINT: z.string(),
  NEXT_PUBLIC_DEFAULT_HOST: z.string(),
  NEXT_PUBLIC_IPFS_AUTH: z.string().optional(),
  NEXT_PUBLIC_IPFS_ENDPOINT: z.string(),
  NEXT_PUBLIC_IPFS_GATEWAY: z.string(),
});

/**
 * You can't destruct `process.env` as a regular object, so you have to do
 * it manually here. This is because Next.js evaluates this at build time,
 * and only used environment variables are included in the build.
 * @type {{ [k in keyof z.infer<typeof clientSchema>]: z.infer<typeof clientSchema>[k] | undefined }}
 */
export const clientEnv = {
  NEXT_PUBLIC_ALCHEMY_ID: process.env.NEXT_PUBLIC_ALCHEMY_ID,
  NEXT_PUBLIC_AXIOM_INGEST_ENDPOINT: process.env.NEXT_PUBLIC_AXIOM_INGEST_ENDPOINT,
  NEXT_PUBLIC_CDN_ENDPOINT: process.env.NEXT_PUBLIC_CDN_ENDPOINT,
  NEXT_PUBLIC_DEFAULT_HOST: process.env.NEXT_PUBLIC_DEFAULT_HOST,
  NEXT_PUBLIC_IPFS_AUTH: process.env.NEXT_PUBLIC_IPFS_AUTH,
  NEXT_PUBLIC_IPFS_ENDPOINT: process.env.NEXT_PUBLIC_IPFS_ENDPOINT,
  NEXT_PUBLIC_IPFS_GATEWAY: process.env.NEXT_PUBLIC_IPFS_GATEWAY,
};
