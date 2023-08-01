import { Profile } from "@wired-protocol/types";

import { HOME_SERVER } from "@/src/constants";
import { parseHandle } from "@/src/utils/parseHandle";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { db } from "../db/drizzle";
import { FixWith } from "../db/types";

export type UserProfile = {
  username: string;
  home: string;
  metadata: Profile;
};

/**
 * Fetches a user's profile given their handle
 */
export async function fetchUserProfile(
  handle: string,
): Promise<UserProfile | null> {
  const { username, home } = parseHandle(handle);
  if (!username || !home) return null;

  if (home === HOME_SERVER) return await fetchUserProfileDB(username);
  else return await fetchUserProfileWired(username, home);
}

/**
 * Fetches a user's profile from the database
 */
export async function fetchUserProfileDB(
  username: string,
): Promise<UserProfile | null> {
  try {
    const _foundUser = await db.query.user.findFirst({
      where: (row, { eq }) => eq(row.username, username),
      with: { profile: true },
    });
    if (!_foundUser) return null;
    const foundUser: FixWith<typeof _foundUser, "profile"> = _foundUser;
    if (!foundUser.profile) return null;

    const background = foundUser.profile.backgroundKey
      ? cdnURL(
        S3Path.profile(foundUser.id).background(
          foundUser.profile.backgroundKey,
        ),
      )
      : undefined;

    const image = foundUser.profile.imageKey
      ? cdnURL(S3Path.profile(foundUser.id).image(foundUser.profile.imageKey))
      : undefined;

    return {
      home: HOME_SERVER,
      metadata: {
        background,
        bio: foundUser.profile.bio ?? undefined,
        image,
        links: [],
      },
      username: foundUser.username,
    };
  } catch {
    return null;
  }
}

/**
 * Fetches a user's profile using the wired protocol
 */
export async function fetchUserProfileWired(
  username: string,
  home: string,
): Promise<UserProfile | null> {
  try {
    const res = await fetch(`${home}/.wired-protocol/v1/users/${username}`, {
      next: { revalidate: 60 },
    });
    if (!res.ok) return null;

    const json = await res.json();
    const metadata = Profile.fromJson(json);

    return {
      home,
      metadata,
      username,
    };
  } catch {
    return null;
  }
}
