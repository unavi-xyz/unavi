import { ATTRIBUTE_TYPES } from "./constants";
import { ERC721Metadata } from "./erc721";

export function getHostFromMetadata(metadata: ERC721Metadata) {
  return metadata.attributes?.find((attr) => attr.trait_type === ATTRIBUTE_TYPES.HOST)?.value;
}
