import { SpaceDBId, SpaceNFTId } from "@/src/utils/parseSpaceId";

import { fetchSpaceNFTMetadata } from "./fetchSpaceNFTMetadata";
import { fetchSpaceNFTOwner } from "./fetchSpaceNFTOwner";
import { readSpaceMetadata, SpaceMetadata } from "./readSpaceMetadata";

export async function validateSpaceNFT(
  tokenId: number,
  owner?: string
): Promise<ValidSpaceNFT | null> {
  try {
    // Check if owned by owner
    if (owner) {
      const spaceOwner = await fetchSpaceNFTOwner(tokenId);
      if (spaceOwner !== owner) throw new Error("Space not owned by owner");
    }

    const erc721metadata = await fetchSpaceNFTMetadata(tokenId);
    if (!erc721metadata?.animation_url) throw new Error("Invalid nft metadata");

    const metadata = await readSpaceMetadata(erc721metadata.animation_url);
    if (!metadata) throw new Error("Invalid space metadata");

    return { id: { type: "tokenId", value: tokenId }, metadata };
  } catch {
    return null;
  }
}

export type ValidSpaceNFT = { id: SpaceNFTId; metadata: SpaceMetadata };
export type ValidDatabaseSpace = { id: SpaceDBId; metadata: SpaceMetadata };
export type ValidSpace = ValidSpaceNFT | ValidDatabaseSpace;
