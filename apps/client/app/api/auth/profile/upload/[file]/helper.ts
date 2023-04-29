import { GetFileUploadResponse, ProfileFile } from "./types";

export async function getProfileUploadURL(file: ProfileFile) {
  const res = await fetch(`/api/auth/profile/upload/${file}`, {
    method: "PUT",
  });

  if (!res.ok) {
    throw new Error("Failed to get upload URL");
  }

  const json = (await res.json()) as GetFileUploadResponse;
  return json;
}
