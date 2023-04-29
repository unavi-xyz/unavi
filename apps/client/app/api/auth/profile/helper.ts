import { UpdateProfileInput } from "./types";

export const ERR_USERNAME_TAKEN = "Username is taken";

export async function updateProfile(args: UpdateProfileInput) {
  const res = await fetch("/api/auth/profile", {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(args),
  });

  if (!res.ok) {
    if (res.status === 409) throw new Error(ERR_USERNAME_TAKEN);
    throw new Error(await res.text());
  }
}
