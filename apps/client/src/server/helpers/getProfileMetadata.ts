import { ERC721MetadataSchema, Profile__factory, PROFILE_ADDRESS } from "contracts";

import { ethersProvider } from "../constants";

export async function getProfileMetadata(profileId: number) {
  const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

  // Fetch metadata uri
  const uri = await contract.tokenURI(profileId);

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
