import { fetchNFTSpaceTokenMetadata } from "./fetchNFTSpaceTokenMetadata";

export async function fetchNFTSpaceURI(id: number) {
  try {
    const erc721metadata = await fetchNFTSpaceTokenMetadata(id);
    if (!erc721metadata?.animation_url) return null;

    return erc721metadata.animation_url;
  } catch {
    return null;
  }
}
