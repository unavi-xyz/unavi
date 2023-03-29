import { fetchProfileFromAddress } from "./fetchProfileFromAddress";
import { fetchSpaceMetadata } from "./fetchSpaceMetadata";
import { fetchSpaceOwner } from "./fetchSpaceOwner";

export async function fetchSpace(id: number): Promise<Space | null> {
  try {
    const metadataPromise = fetchSpaceMetadata(id);
    const owner = await fetchSpaceOwner(id);
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
  owner: Awaited<ReturnType<typeof fetchSpaceOwner>>;
  profile: Awaited<ReturnType<typeof fetchProfileFromAddress>>;
  metadata: Awaited<ReturnType<typeof fetchSpaceMetadata>>;
};
