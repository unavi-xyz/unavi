import { GetTempResponse } from "./types";

export async function getTempUpload() {
  const response = await fetch("/api/temp", { method: "POST" });
  const data = (await response.json()) as GetTempResponse;
  return data;
}
