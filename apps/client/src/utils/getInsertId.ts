import { ExecutedQuery } from "@planetscale/database";
import { MySqlRawQueryResult } from "drizzle-orm/mysql2";

export function getInsertId(
  result: MySqlRawQueryResult | ExecutedQuery
): number {
  if ("insertId" in result) return parseInt(result.insertId);
  if ("insertId" in result[0]) return result[0].insertId;
  throw new Error("Could not get insert ID");
}
