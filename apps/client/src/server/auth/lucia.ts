import "lucia/polyfill/node";

import { lucia } from "lucia";
import { nextjs } from "lucia/middleware";

import { env } from "@/src/env.mjs";

import {
  AUTH_KEY_TABLE_NAME,
  AUTH_SESSION_TABLE_NAME,
  AUTH_USER_TABLE_NAME,
} from "../db/constants";
import { mysql2Connection, planetscaleConnection } from "../db/drizzle";
import { SESSION_COOKIE_NAME } from "./constants";

async function getPlanetscaleAdapter() {
  const { planetscale } = await import("@lucia-auth/adapter-mysql");
  return planetscale(planetscaleConnection, {
    key: AUTH_KEY_TABLE_NAME,
    session: AUTH_SESSION_TABLE_NAME,
    user: AUTH_USER_TABLE_NAME,
  });
}

async function getMysql2Adapter() {
  const { mysql2 } = await import("@lucia-auth/adapter-mysql");
  return mysql2(mysql2Connection, {
    key: AUTH_KEY_TABLE_NAME,
    session: AUTH_SESSION_TABLE_NAME,
    user: AUTH_USER_TABLE_NAME,
  });
}

export const luciaEnv = process.env.NODE_ENV === "development" ? "DEV" : "PROD";

const adapter = env.PLANETSCALE
  ? await getPlanetscaleAdapter()
  : await getMysql2Adapter();

export const auth = lucia({
  adapter,
  env: luciaEnv,
  getUserAttributes: (data) => {
    return {
      address: data.address,
      did: data.did,
      username: data.username,
    };
  },
  middleware: nextjs(),
  sessionCookie: {
    expires: false,
    name: SESSION_COOKIE_NAME,
  },
});

export type Auth = typeof auth;
