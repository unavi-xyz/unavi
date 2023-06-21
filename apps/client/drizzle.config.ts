import type { Config } from "drizzle-kit";

if (!process.env.DATABASE_URL) {
  throw new Error("DATABASE_URL is missing");
}

export default {
  connectionString: process.env.DATABASE_URL,
  schema: "./src/server/db/schema.ts",
} satisfies Config;
