import { WorldMetadata } from "@wired-protocol/types";

import { env } from "@/src/env.mjs";

import { db } from "../db/drizzle";
import { FixWith } from "../db/types";
import { getWorldJson } from "./getWorldJson";

export async function fetchLatestWorlds(limit: number, ownerId?: string) {
  if (!env.NEXT_PUBLIC_HAS_DATABASE) return [];

  try {
    const _worlds = await db.query.world.findMany({
      columns: { publicId: true },
      limit,
      where: ownerId ? (row, { eq }) => eq(row.ownerId, ownerId) : undefined,
      with: {
        model: {
          columns: {
            key: true,
          },
        },
      },
    });
    const worlds: FixWith<(typeof _worlds)[0], "model">[] = _worlds;

    const fetched: FetchedWorld[] = [];

    await Promise.all(
      worlds.map(async (world) => {
        if (!world.model) return;

        const metadata = await getWorldJson(world.publicId);
        if (!metadata) return;

        fetched.push({
          id: world.publicId,
          metadata,
          uri: `${env.NEXT_PUBLIC_DEPLOYED_URL}/api/worlds/${world.publicId}`,
        });
      })
    );

    return fetched;
  } catch (e) {
    console.error(e);
    return [];
  }
}

export type FetchedWorld = {
  id: string;
  metadata: WorldMetadata;
  uri: string;
};
