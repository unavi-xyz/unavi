export function parseSpaceId(id: string): SpaceId {
  // If hex, assume it's a token id
  if (id.match("0x[0-9a-fA-F]+")) return { type: "tokenId", value: parseInt(id) };
  // If no protocol, assume it's a db id
  if (!id.match("^[a-z]+://")) return { type: "id", value: id };
  // Otherwise, assume it's a uri
  return { type: "uri", value: id };
}

export type SpaceNFTId = { type: "tokenId"; value: number };
export type SpaceDBId = { type: "id"; value: string };
export type SpaceURIId = { type: "uri"; value: string };

export type SpaceId = SpaceNFTId | SpaceDBId | SpaceURIId;
