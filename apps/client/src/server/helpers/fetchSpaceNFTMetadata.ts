import { ERC721MetadataSchema, Space__factory, SPACE_ADDRESS } from "contracts";
import { cache } from "react";

import { ethersProvider } from "../ethers";

export const fetchSpaceNFTMetadata = cache(async (id: number) => {
  try {
    const contract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

    // Fetch metadata uri
    const uri = await contract.tokenURI(id);

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
