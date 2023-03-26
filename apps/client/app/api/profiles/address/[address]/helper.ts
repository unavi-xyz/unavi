import { Profile } from "../../../../../src/server/helpers/fetchProfile";

export async function getProfileByAddress(address: string) {
  const response = await fetch(`/api/profiles/address/${address}`, {
    method: "GET",
    cache: "force-cache",
  });
  const profile = (await response.json()) as Profile | null;
  return profile;
}
