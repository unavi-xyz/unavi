import { PostMintResponse } from "./types";

export async function mintSpace(spaceId: string) {
  const res = await fetch(`/api/spaces/${spaceId}/mint`, { method: "POST" });
  if (!res.ok) throw new Error(await res.text());

  return (await res.json()) as PostMintResponse;
}
