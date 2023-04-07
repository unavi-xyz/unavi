import { ERC721Metadata } from "contracts";

import { Profile } from "./fetchProfile";
import { fetchProfileFromAddress } from "./fetchProfileFromAddress";
import { fetchSpaceNFTMetadata } from "./fetchSpaceNFTMetadata";
import { fetchSpaceNFTOwner } from "./fetchSpaceNFTOwner";

export async function fetchSpaceNFT(id: number): Promise<Space | null> {
  try {
    const metadataPromise = fetchSpaceNFTMetadata(id);
    const owner = await fetchSpaceNFTOwner(id);
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
