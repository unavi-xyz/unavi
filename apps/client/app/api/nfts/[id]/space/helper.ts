import { GetNftSpaceResponse } from "./types";

export async function getNFTSpace(nftId: string) {
  const response = await fetch(`/api/nfts/${nftId}/space`, { method: "GET" });
  return (await response.json()) as GetNftSpaceResponse;
}
