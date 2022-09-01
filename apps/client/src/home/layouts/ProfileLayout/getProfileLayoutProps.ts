import {
  HANDLE_ENDING,
  getMediaImageSSR,
  getMediaImageUri,
} from "@wired-xr/lens";
import {
  GetProfileDocument,
  GetProfileQuery,
  GetProfileQueryVariables,
  Profile,
} from "@wired-xr/lens";

import { lensClient } from "../../../lib/lens/client";
import { PageMetadata } from "../../../types";
import { parseUri } from "../../../utils/parseUri";

export interface ProfileLayoutProps {
  handle: string;
  metadata: PageMetadata;
  profile: Profile | undefined;
  coverImage: string | null;
  profileImage: string | null;
}

export async function getProfileLayoutProps(
  handle: string
): Promise<ProfileLayoutProps> {
  const profileQuery = await lensClient
    .query<GetProfileQuery, GetProfileQueryVariables>(GetProfileDocument, {
      request: {
        handles: [handle.concat(HANDLE_ENDING)],
      },
    })
    .toPromise();

  const profile = profileQuery.data?.profiles.items[0] as Profile;
  const title = profile?.name ? `${profile?.name} (@${handle})` : `@${handle}`;
  const metadata: PageMetadata = {
    title,
    description: profile?.bio ?? "",
    image: getMediaImageSSR(profile?.picture) ?? "",
  };

  const coverImage = profile?.coverPicture
    ? parseUri(getMediaImageUri(profile.coverPicture))
    : null;
  const profileImage = profile?.picture
    ? parseUri(getMediaImageUri(profile.picture))
    : null;

  return { handle, metadata, profile, coverImage, profileImage };
}
