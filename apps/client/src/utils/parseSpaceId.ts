export type SpaceNFTId = { type: "tokenId"; value: number };
export type SpaceDBId = { type: "id"; value: string };
export type SpaceId = SpaceNFTId | SpaceDBId;

export function parseSpaceId(id: string): SpaceId {
  if (id.startsWith("0x")) {
    return { type: "tokenId", value: parseInt(id) };
  } else {
    return { type: "id", value: id };
  }
}
