import { ERC721MetadataSchema, Space__factory, SPACE_ADDRESS } from "contracts";
import { cache } from "react";

import { ethersProvider } from "../constants";

export const fetchSpaceMetadata = cache(async (spaceId: number) => {
  const contract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

  // Fetch metadata uri
  const uri = await contract.tokenURI(spaceId);

  // No uri found
  if (!uri) return null;

  try {
    const res = await fetch(uri, { next: { revalidate: 60 } });
    const data = await res.json();
    const metadata = ERC721MetadataSchema.parse(data);

    return metadata;
  } catch {
    return null;
  }
});
