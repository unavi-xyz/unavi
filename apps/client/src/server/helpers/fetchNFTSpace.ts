import { ERC721Metadata } from "contracts";

import { fetchNFTSpaceOwner } from "./fetchNFTSpaceOwner";
import { fetchNFTSpaceTokenMetadata } from "./fetchNFTSpaceTokenMetadata";

export async function fetchNFTSpace(id: number): Promise<Space | null> {
  try {
    const metadataPromise = fetchNFTSpaceTokenMetadata(id);
    const owner = await fetchNFTSpaceOwner(id);
    const metadata = await metadataPromise;

    return {
      id,
      metadata,
      owner,
    };
  } catch {
    return null;
  }
}

export type Space = {
  id: number;
  owner: string;
  metadata: ERC721Metadata | null;
};
