import { fetchProfileFromAddress } from "./fetchProfileFromAddress";
import { fetchSpaceMetadata } from "./fetchSpaceMetadata";
import { fetchSpaceOwner } from "./fetchSpaceOwner";

export async function fetchSpace(id: number) {
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
}
