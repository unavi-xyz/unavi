import type { Config } from "drizzle-kit";

if (!process.env.DATABASE_URL) {
  throw new Error("DATABASE_URL is missing");
}

const dbUrl = process.env.DATABASE_URL;
const secureUrl = dbUrl?.replace(
  "?sslaccept=strict",
  `?ssl={"rejectUnauthorized":true}`
);

export default {
  connectionString: secureUrl,
  schema: "./src/server/db/schema.ts",
} satisfies Config;
