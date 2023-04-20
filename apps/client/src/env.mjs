import { z } from "zod";

/**
 * Specify your server-side environment variables schema here. This way you can ensure the app isn't
 * built with invalid env vars.
 */
const server = z.object({
  DATABASE_URL: z.string().url().optional(),
  DISABLE_PWA: z.string().optional(),
  ETH_PROVIDER: z.string().url(),
  NODE_ENV: z.enum(["development", "test", "production"]),
  NEXTAUTH_SECRET:
    process.env.NODE_ENV === "production" ? z.string().min(1) : z.string().min(1).optional(),
  NEXTAUTH_URL: z.preprocess(
    // This makes Vercel deployments not fail if you don't set NEXTAUTH_URL
    // Since NextAuth.js automatically uses the VERCEL_URL if present.
    (str) => str ?? process.env.VERCEL_URL,
    // VERCEL_URL doesn't include `https` so it cant be validated as a URL
    process.env.VERCEL ? z.string().min(1) : z.string().url()
  ),
  S3_ACCESS_KEY_ID: z.string().optional(),
  S3_BUCKET: z.string().optional(),
  S3_ENDPOINT: z.string().optional(),
  S3_REGION: z.string().optional(),
  S3_SECRET: z.string().optional(),
  VERCEL_URL: z.string().optional(),
});

/**
 * Specify your client-side environment variables schema here. This way you can ensure the app isn't
 * built with invalid env vars. To expose them to the client, prefix them with `NEXT_PUBLIC_`.
 */
const client = z.object({
  NEXT_PUBLIC_ALCHEMY_ID: z.string().optional(),
  NEXT_PUBLIC_AXIOM_INGEST_ENDPOINT: z.string().optional(),
  NEXT_PUBLIC_CDN_ENDPOINT: z.string().optional(),
  NEXT_PUBLIC_CRYPTOAVATARS_API_KEY: z.string().optional(),
  NEXT_PUBLIC_DEFAULT_HOST: z.string(),
  NEXT_PUBLIC_DEPLOYED_URL: z.string().url(),
  NEXT_PUBLIC_DOCS_URL: z.string().url(),
  NEXT_PUBLIC_HAS_DATABASE: z.boolean(),
  NEXT_PUBLIC_HAS_S3: z.boolean(),
});

/**
 * You can't destruct `process.env` as a regular object in the Next.js edge runtimes (e.g.
 * middlewares) or client-side so we need to destruct manually.
 *
 * @type {Record<keyof z.infer<typeof server> | keyof z.infer<typeof client>, string | undefined>}
 */
const processEnv = {
  DATABASE_URL: process.env.DATABASE_URL,
  DISABLE_PWA: process.env.DISABLE_PWA,
  ETH_PROVIDER: process.env.ETH_PROVIDER,
  NODE_ENV: process.env.NODE_ENV,
  NEXTAUTH_SECRET: process.env.NEXTAUTH_SECRET,
  NEXTAUTH_URL: process.env.NEXTAUTH_URL,
  S3_ACCESS_KEY_ID: process.env.S3_ACCESS_KEY_ID,
  S3_BUCKET: process.env.S3_BUCKET,
  S3_ENDPOINT: process.env.S3_ENDPOINT,
  S3_REGION: process.env.S3_REGION,
  S3_SECRET: process.env.S3_SECRET,
  NEXT_PUBLIC_ALCHEMY_ID: process.env.NEXT_PUBLIC_ALCHEMY_ID,
  NEXT_PUBLIC_AXIOM_INGEST_ENDPOINT: process.env.NEXT_PUBLIC_AXIOM_INGEST_ENDPOINT,
  NEXT_PUBLIC_CDN_ENDPOINT: process.env.NEXT_PUBLIC_CDN_ENDPOINT,
  NEXT_PUBLIC_CRYPTOAVATARS_API_KEY: process.env.NEXT_PUBLIC_CRYPTOAVATARS_API_KEY,
  NEXT_PUBLIC_DEFAULT_HOST: process.env.NEXT_PUBLIC_DEFAULT_HOST,
  NEXT_PUBLIC_DEPLOYED_URL: process.env.NEXT_PUBLIC_DEPLOYED_URL,
  NEXT_PUBLIC_DOCS_URL: process.env.NEXT_PUBLIC_DOCS_URL,
  NEXT_PUBLIC_HAS_DATABASE: process.env.NEXT_PUBLIC_HAS_DATABASE === "true",
  NEXT_PUBLIC_HAS_S3: process.env.NEXT_PUBLIC_HAS_S3 === "true",
  VERCEL_URL: process.env.VERCEL_URL,
};

// Don't touch the part below
// --------------------------

const merged = server.merge(client);

/** @typedef {z.input<typeof merged>} MergedInput */
/** @typedef {z.infer<typeof merged>} MergedOutput */
/** @typedef {z.SafeParseReturnType<MergedInput, MergedOutput>} MergedSafeParseReturn */

let env = /** @type {MergedOutput} */ (process.env);

const skip =
  !!process.env.SKIP_ENV_VALIDATION &&
  process.env.SKIP_ENV_VALIDATION !== "false" &&
  process.env.SKIP_ENV_VALIDATION !== "0";
if (!skip) {
  const isServer = typeof window === "undefined";

  const parsed = /** @type {MergedSafeParseReturn} */ (
    isServer
      ? merged.safeParse(processEnv) // on server we can validate all env vars
      : client.safeParse(processEnv) // on client we can only validate the ones that are exposed
  );

  if (parsed.success === false) {
    console.error("❌ Invalid environment variables:", parsed.error.flatten().fieldErrors);
    throw new Error("Invalid environment variables");
  }

  // eslint-disable-next-line
  env = new Proxy(parsed.data, {
    get(target, prop) {
      if (typeof prop !== "string") return undefined;
      // Throw a descriptive error if a server-side env var is accessed on the client
      // Otherwise it would just be returning `undefined` and be annoying to debug
      if (!isServer && !prop.startsWith("NEXT_PUBLIC_"))
        throw new Error(
          process.env.NODE_ENV === "production"
            ? "❌ Attempted to access a server-side environment variable on the client"
            : `❌ Attempted to access server-side environment variable '${prop}' on the client`
        );
      return target[/** @type {keyof typeof target} */ (prop)];
    },
  });
}

export { env };
