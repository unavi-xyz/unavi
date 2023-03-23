import { ERC721MetadataSchema, Profile__factory, PROFILE_ADDRESS } from "contracts";
import { cache } from "react";

import { ethersProvider } from "../ethers";

export const fetchProfileMetadata = cache(async (profileId: number) => {
  try {
    const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

    // Fetch metadata uri
    const uri = await contract.tokenURI(profileId);

    // No uri found
    if (!uri) return null;

    const res = await fetch(uri, { next: { revalidate: 60 } });
    const data = await res.json();
    const metadata = ERC721MetadataSchema.parse(data);

    return metadata;
  } catch {
    return null;
  }
});
