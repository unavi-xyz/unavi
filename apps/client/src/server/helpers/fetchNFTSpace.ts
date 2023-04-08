import { ERC721Metadata } from "contracts";

import { fetchNFTSpaceOwner } from "./fetchNFTSpaceOwner";
import { fetchNFTSpaceTokenMetadata } from "./fetchNFTSpaceTokenMetadata";
import { Profile } from "./fetchProfile";
import { fetchProfileFromAddress } from "./fetchProfileFromAddress";

export async function fetchNFTSpace(id: number): Promise<Space | null> {
  try {
    const metadataPromise = fetchNFTSpaceTokenMetadata(id);
    const owner = await fetchNFTSpaceOwner(id);
    const profile = await fetchProfileFromAddress(owner);
    const metadata = await metadataPromise;

    return {
      id,
      owner,
      profile,
      metadata,
    };
  } catch {
    return null;
  }
}

export type Space = {
  id: number;
  owner: string;
  profile: Profile | null;
  metadata: ERC721Metadata | null;
};
