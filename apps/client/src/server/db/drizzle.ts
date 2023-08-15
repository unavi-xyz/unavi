import type { Connection } from "@planetscale/database";
import { MySql2Database } from "drizzle-orm/mysql2";
import { PlanetScaleDatabase } from "drizzle-orm/planetscale-serverless";
import type { Pool } from "mysql2/promise";

import { env } from "@/src/env.mjs";

import * as schema from "./schema";

const LOG = env.NODE_ENV === "development";

export let planetscaleConnection: Connection;
export let mysql2Connection: Pool;

export let db:
  | PlanetScaleDatabase<typeof schema>
  | MySql2Database<typeof schema>;

const dbUrl = env.DATABASE_URL;
const secureUrl = dbUrl?.replace(
  "?sslaccept=strict",
  `?ssl={"rejectUnauthorized":true}`
);

if (env.PLANETSCALE) {
  const { connect } = await import("@planetscale/database");
  const { drizzle } = await import("drizzle-orm/planetscale-serverless");

  planetscaleConnection = connect({ url: secureUrl });

  db = drizzle(planetscaleConnection, { logger: LOG, schema });
} else {
  const { createPool } = await import("mysql2/promise");
  const { drizzle } = await import("drizzle-orm/mysql2");

  mysql2Connection = createPool({ uri: secureUrl });

  db = drizzle(mysql2Connection, { logger: LOG, mode: "planetscale", schema });
}
