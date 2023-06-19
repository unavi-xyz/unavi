export function parseWorldId(id: string): SpaceId {
  // If no protocol, assume it's a db id
  if (!id.match("^[a-z]+://")) return { type: "id", value: id };
  // Otherwise, assume it's a uri
  return { type: "uri", value: id };
}

export type WorldDBId = { type: "id"; value: string };
export type WorldURIId = { type: "uri"; value: string };

export type SpaceId = WorldDBId | WorldURIId;
