import { WorldMetadata } from "@wired-protocol/types";

import { env } from "@/src/env.mjs";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { db } from "../db/drizzle";
import { fetchWorldMetadata } from "./fetchWorldMetadata";

export async function fetchLatestWorlds(limit: number, ownerId?: string) {
  if (!env.NEXT_PUBLIC_HAS_DATABASE) return [];

  try {
    const worlds = await db.query.world.findMany({
      columns: {
        name: true,
        publicId: true,
      },
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

    const fetched: FetchedWorld[] = [];

    const fetchWorld = async (world: (typeof worlds)[0]) => {
      const uri = cdnURL(S3Path.worldModel(world.model.key).metadata);

      const json = await fetchWorldMetadata(uri);
      if (!json) return;

      fetched.push({
        id: world.publicId,
        metadata: json.metadata,
        uri: json.uri,
      });
    };

    await Promise.all(worlds.map(fetchWorld));

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
