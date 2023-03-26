import { ERC721MetadataSchema, Space__factory, SPACE_ADDRESS } from "contracts";

import { ethersProvider } from "./ethers";

export async function fetchMetadata(spaceId: number) {
  try {
    const contract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

    // Fetch metadata uri
    const uri = await contract.tokenURI(spaceId);

    // No uri found
    if (!uri) return null;

    const res = await fetch(uri, { next: { revalidate: 60 } });
    const data = await res.json();
    const metadata = ERC721MetadataSchema.parse(data);

    return metadata;
  } catch {
    return null;
  }
}
