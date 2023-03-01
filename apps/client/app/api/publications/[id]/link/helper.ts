import { PutParams } from "./types";

export function linkPublication(id: string, spaceId: number) {
  const json: PutParams = { spaceId };
  return fetch(`/api/publications/${id}/link`, { method: "PUT", body: JSON.stringify(json) });
}
