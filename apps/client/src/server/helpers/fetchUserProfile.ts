import { ProfileMetadata, ProfileMetadataSchema } from "@wired-protocol/types";

import { env } from "@/src/env.mjs";
import { parseHandle } from "@/src/utils/parseHandle";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import { db } from "../db/drizzle";
import { FixWith } from "../db/types";

export type UserProfile = {
  username: string;
  domain: string;
  metadata: ProfileMetadata;
};

/**
 * Fetches a user's profile given their handle
 */
export async function fetchUserProfile(
  handle: string
): Promise<UserProfile | null> {
  const { username, domain } = parseHandle(handle);
  if (!username || !domain) return null;

  if (domain === env.NEXT_PUBLIC_DEPLOYED_URL)
    return await fetchUserProfileDB(username);
  else return await fetchUserProfileWired(username, domain);
}

/**
 * Fetches a user's profile from the database
 */
export async function fetchUserProfileDB(
  username: string
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
            foundUser.profile.backgroundKey
          )
        )
      : undefined;
    const image = foundUser.profile.imageKey
      ? cdnURL(S3Path.profile(foundUser.id).image(foundUser.profile.imageKey))
      : undefined;

    return {
      domain: env.NEXT_PUBLIC_DEPLOYED_URL,
      metadata: {
        background,
        bio: foundUser.profile.bio ?? undefined,
        image,
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
  domain: string
): Promise<UserProfile | null> {
  try {
    const res = await fetch(`${domain}/.wired-protocol/v1/users/${username}`, {
      next: { revalidate: 60 },
    });
    if (!res.ok) return null;

    const parsed = ProfileMetadataSchema.safeParse(await res.json());

    if (!parsed.success) return null;

    return {
      domain,
      metadata: parsed.data,
      username,
    };
  } catch {
    return null;
  }
}
