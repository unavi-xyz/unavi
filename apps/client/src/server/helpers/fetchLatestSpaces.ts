import { readContract } from "wagmi/actions";

import { SPACE_ADDRESS } from "@/src/contracts/addresses";
import { SPACE_ABI } from "@/src/contracts/SpaceAbi";
import { env } from "@/src/env.mjs";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { prisma } from "../prisma";
import { fetchWorldMetadata } from "./fetchWorldMetadata";
import {
  validateSpaceNFT,
  ValidDBSpace,
  ValidNFTSpace,
} from "./validateSpaceNFT";

export async function fetchLatestSpaces(limit: number, owner?: string) {
  const [nftSpaces, databaseSpaces] = await Promise.all([
    fetchNFTSpaces(limit, owner),
    fetchDatabaseSpaces(limit, owner),
  ]);

  return [...nftSpaces, ...databaseSpaces];
}

async function fetchNFTSpaces(limit: number, owner?: string) {
  try {
    const count = Number(
      await readContract({
        abi: SPACE_ABI,
        address: SPACE_ADDRESS,
        functionName: "count",
      })
    );

    const spaces: ValidNFTSpace[] = [];
    const length = Math.min(limit, count);
    let nextSpaceId = count - 1;

    const fetchSpace = async () => {
      if (nextSpaceId === 0 || spaces.length === length) return;

      const valid = await validateSpaceNFT(nextSpaceId--, owner);

      if (valid) spaces.push(valid);
      else await fetchSpace();
    };

    await Promise.all(Array.from({ length }).map(fetchSpace));

    return spaces.sort((a, b) => b.id.value - a.id.value);
  } catch {
    return [];
  }
}

export async function fetchDatabaseSpaces(limit: number, owner?: string) {
  if (!env.NEXT_PUBLIC_HAS_DATABASE) return [];

  try {
    const spaces = await prisma.space.findMany({
      include: { SpaceModel: true },
      orderBy: { updatedAt: "desc" },
      take: limit,
      where: {
        Owner: { username: owner },
        SpaceModel: { isNot: null },
        tokenId: null,
      },
    });

    const validSpaces: ValidDBSpace[] = [];

    await Promise.all(
      spaces.map(async (space) => {
        if (!space.SpaceModel) return;

        const uri = cdnURL(
          S3Path.spaceModel(space.SpaceModel.publicId).metadata
        );
        const world = await fetchWorldMetadata(uri);
        if (!world) return;

        validSpaces.push({
          id: { type: "id", value: space.publicId },
          metadata: world.metadata,
          uri,
        });
      })
    );

    return validSpaces;
  } catch {
    return [];
  }
}
