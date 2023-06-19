import { eq } from "drizzle-orm";
import { readContract } from "wagmi/actions";

import { SPACE_ADDRESS } from "@/src/contracts/addresses";
import { SPACE_ABI } from "@/src/contracts/SpaceAbi";
import { env } from "@/src/env.mjs";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { db } from "../db/drizzle";
import { world } from "../db/schema";
import { fetchWorldMetadata } from "./fetchWorldMetadata";
import {
  validateWorldNFT,
  ValidDBWorld,
  ValidNFTWorld,
} from "./validateWorldNFT";

export async function fetchLatestWorlds(limit: number) {
  const [nftSpaces, databaseSpaces] = await Promise.all([
    fetchNFTWorlds(limit),
    fetchDatabaseWorlds(limit),
  ]);

  return [...nftSpaces, ...databaseSpaces];
}

export async function fetchNFTWorlds(limit: number, ownerAddress?: string) {
  try {
    const count = Number(
      await readContract({
        abi: SPACE_ABI,
        address: SPACE_ADDRESS,
        functionName: "count",
      })
    );

    const valid: ValidNFTWorld[] = [];
    const length = Math.min(limit, count);
    let nextSpaceId = count - 1;

    const fetchSpace = async () => {
      if (nextSpaceId === 0 || valid.length === length) return;

      const found = await validateWorldNFT(nextSpaceId--, ownerAddress);

      if (found) valid.push(found);
      else await fetchSpace();
    };

    await Promise.all(Array.from({ length }).map(fetchSpace));

    return valid.sort((a, b) => b.id.value - a.id.value);
  } catch {
    return [];
  }
}

export async function fetchDatabaseWorlds(limit: number, ownerId?: string) {
  if (!env.NEXT_PUBLIC_HAS_DATABASE) return [];

  try {
    const worlds = await db.query.world.findMany({
      columns: {
        name: true,
        publicId: true,
      },
      limit,
      where: ownerId ? eq(world.ownerId, ownerId) : undefined,
      with: {
        model: {
          columns: {
            key: true,
          },
        },
      },
    });

    const valid: ValidDBWorld[] = [];

    const fetchWorld = async (world: (typeof worlds)[0]) => {
      const uri = cdnURL(S3Path.worldModel(world.model.key).metadata);

      const json = await fetchWorldMetadata(uri);
      if (!json) return;

      valid.push({
        id: { type: "id", value: world.publicId },
        metadata: json.metadata,
        uri: json.uri,
      });
    };

    await Promise.all(worlds.map(fetchWorld));

    return valid;
  } catch {
    return [];
  }
}
