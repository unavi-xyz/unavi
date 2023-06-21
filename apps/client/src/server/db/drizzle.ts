import type { Connection } from "@planetscale/database";
import { MySql2Database } from "drizzle-orm/mysql2";
import { PlanetScaleDatabase } from "drizzle-orm/planetscale-serverless";
import type { Pool } from "mysql2/promise";

import { env } from "@/src/env.mjs";

import * as schema from "./schema";

const PLANETSCALE = false;
const LOG = env.NODE_ENV === "development";

export let planetscaleConnection: Connection;
export let mysql2Connection: Pool;

export let db:
  | PlanetScaleDatabase<typeof schema>
  | MySql2Database<typeof schema>;

if (PLANETSCALE) {
  const { connect } = await import("@planetscale/database");
  const { drizzle } = await import("drizzle-orm/planetscale-serverless");

  planetscaleConnection = connect({ url: env.DATABASE_URL });
  db = drizzle(planetscaleConnection, { logger: LOG, schema });
} else {
  const { createPool } = await import("mysql2/promise");
  const { drizzle } = await import("drizzle-orm/mysql2");

  mysql2Connection = createPool({ uri: env.DATABASE_URL });
  db = drizzle(mysql2Connection, { logger: LOG, schema });
}
