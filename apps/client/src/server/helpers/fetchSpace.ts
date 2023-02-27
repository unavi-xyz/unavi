import { fetchSpaceMetadata } from "./fetchSpaceMetadata";
import { fetchSpaceOwner } from "./fetchSpaceOwner";

export async function fetchSpace(id: number) {
  const [{ address, profile }, metadata] = await Promise.all([
    fetchSpaceOwner(id),
    fetchSpaceMetadata(id),
  ]);

  return {
    id,
    owner: address,
    author: profile,
    metadata,
  };
}
