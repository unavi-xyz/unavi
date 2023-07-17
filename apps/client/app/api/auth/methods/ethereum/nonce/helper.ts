import { GetNonceResponse } from "./types";

export async function getNonce() {
  const res = await fetch("/api/auth/methods/ethereum/nonce", {
    method: "GET",
  });
  const { nonce } = (await res.json()) as GetNonceResponse;
  return nonce;
}
