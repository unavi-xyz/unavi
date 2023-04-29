import { GetAuthStatusResponse } from "./types";

export async function getAuthStatus() {
  const res = await fetch("/api/auth/status");
  const json = (await res.json()) as GetAuthStatusResponse;
  return json;
}
