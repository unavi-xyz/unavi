import { WorldMetadata } from "@wired-protocol/types";

import { SpaceDBId, SpaceNFTId } from "@/src/utils/parseSpaceId";

import { fetchNFTSpaceOwner } from "./fetchNFTSpaceOwner";
import { fetchNFTSpaceURI } from "./fetchNFTSpaceURI";
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

    const uri = await fetchNFTSpaceURI(tokenId);
    if (!uri) throw new Error("No space URI");

    const world = await fetchWorldMetadata(uri);
    if (!world) throw new Error("Invalid world metadata");

    return {
      id: { type: "tokenId", value: tokenId },
      metadata: world.metadata,
      uri,
    };
  } catch {
    return null;
  }
}

export type ValidNFTSpace = { id: SpaceNFTId; uri: string; metadata: WorldMetadata };
export type ValidDBSpace = { id: SpaceDBId; uri: string; metadata: WorldMetadata };
export type ValidSpace = ValidNFTSpace | ValidDBSpace;
