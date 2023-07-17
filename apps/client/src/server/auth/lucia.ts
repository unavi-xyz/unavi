import "lucia-auth/polyfill/node";

import lucia from "lucia-auth";
import { nextjs } from "lucia-auth/middleware";

import { env } from "@/src/env.mjs";

import { mysql2Connection, planetscaleConnection } from "../db/drizzle";

export const luciaEnv = process.env.NODE_ENV === "development" ? "DEV" : "PROD";

const adapter = env.PLANETSCALE
  ? await getPlanetscaleAdapter()
  : await getMysql2Adapter();

export const auth = lucia({
  adapter,
  env: luciaEnv,
  middleware: nextjs(),
  transformDatabaseUser: (userData) => {
    return {
      address: userData.address,
      userId: userData.id,
      username: userData.username,
    };
  },
});

export type Auth = typeof auth;

async function getPlanetscaleAdapter() {
  const { planetscale } = await import("@lucia-auth/adapter-mysql");
  return planetscale(planetscaleConnection);
}

async function getMysql2Adapter() {
  const { mysql2 } = await import("@lucia-auth/adapter-mysql");
  return mysql2(mysql2Connection);
}
