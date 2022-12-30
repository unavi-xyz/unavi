import { ERC721MetadataSchema, Space__factory, SPACE_ADDRESS } from "contracts";

import { ethersProvider } from "../constants";

export async function getSpaceMetadata(spaceId: number) {
  const contract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

  // Fetch metadata uri
  const uri = await contract.tokenURI(spaceId);

  // No uri found
  if (!uri) return null;

  try {
    const res = await fetch(uri);
    const data = await res.json();
    const metadata = ERC721MetadataSchema.parse(data);

    return metadata;
  } catch {
    return null;
  }
}
