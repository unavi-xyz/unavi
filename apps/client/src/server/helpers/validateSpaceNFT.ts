import { WorldMetadata } from "@wired-protocol/types";

import { SpaceDBId, SpaceNFTId } from "@/src/utils/parseSpaceId";

import { fetchNFTSpaceOwner } from "./fetchNFTSpaceOwner";
import { fetchNFTSpaceTokenMetadata } from "./fetchNFTSpaceTokenMetadata";
import { fetchWorldMetadata } from "./fetchWorldMetadata";

export async function validateSpaceNFT(
  tokenId: number,
  owner?: string
): Promise<ValidNFTSpace | null> {
  try {
    // Check if owned by owner
    if (owner) {
      const spaceOwner = await fetchNFTSpaceOwner(tokenId);
      if (spaceOwner !== owner) throw new Error("Space not owned by owner");
    }

    const erc721metadata = await fetchNFTSpaceTokenMetadata(tokenId);
    if (!erc721metadata?.animation_url) throw new Error("Invalid nft metadata");

    const metadata = await fetchWorldMetadata(erc721metadata.animation_url);
    if (!metadata) throw new Error("Invalid space metadata");

    return { id: { type: "tokenId", value: tokenId }, metadata };
  } catch {
    return null;
  }
}

export type ValidNFTSpace = { id: SpaceNFTId; metadata: WorldMetadata };
export type ValidDBSpace = { id: SpaceDBId; metadata: WorldMetadata };
export type ValidSpace = ValidNFTSpace | ValidDBSpace;
