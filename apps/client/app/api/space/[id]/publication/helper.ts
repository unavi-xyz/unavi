import { GetSpacePublicationResponse } from "./types";

export async function getSpacePublication(id: number) {
  const response = await fetch(`/api/space/${id}/publication`, { method: "GET" });
  const publication = (await response.json()) as GetSpacePublicationResponse;
  return publication;
}
