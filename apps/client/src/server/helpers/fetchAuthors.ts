import { World } from "@wired-protocol/types";

import { fetchUserProfile, UserProfile } from "./fetchUserProfile";

export async function fetchAuthors(metadata: World) {
  if (!metadata?.authors) return [];

  const profiles: UserProfile[] = [];

  await Promise.all(
    metadata.authors.map(async (author) => {
      const profile = await fetchUserProfile(author);

      if (!profile) {
        profiles.push({
          home: "",
          metadata: { links: [], name: author },
          username: "",
        });
        return;
      }

      profiles.push(profile);
    }),
  );

  return profiles;
}
