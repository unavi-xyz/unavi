import { WorldMetadata } from "@wired-protocol/types";

import { fetchUserProfile, UserProfile } from "./fetchUserProfile";

export async function fetchAuthors(metadata: WorldMetadata) {
  if (!metadata.info?.authors) return [];

  const profiles: UserProfile[] = [];

  await Promise.all(
    metadata.info.authors.map(async (author) => {
      const profile = await fetchUserProfile(author);

      if (!profile) {
        profiles.push({
          home: "",
          metadata: { name: author },
          username: "",
        });
        return;
      }

      profiles.push(profile);
    })
  );

  return profiles;
}
