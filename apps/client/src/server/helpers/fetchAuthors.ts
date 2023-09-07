import { World } from "@wired-protocol/types";

import { parseIdentity } from "@/src/utils/parseIdentity";

import { fetchProfile, IdentityProfile } from "./fetchProfile";

export async function fetchAuthors(metadata: World) {
  if (!metadata?.authors) return [];

  const profiles: Array<IdentityProfile | string> = [];

  await Promise.all(
    metadata.authors.map(async (author) => {
      const identity = parseIdentity(author);
      const profile = await fetchProfile(identity);

      if (profile) {
        profiles.push(profile);
      } else {
        profiles.push(author);
      }
    })
  );

  return profiles;
}
