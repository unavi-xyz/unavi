import {
  GetProfileDocument,
  GetProfileQuery,
  GetProfileQueryVariables,
  Profile,
} from "@wired-labs/lens";

import { lensClient } from "../../../lib/lens/client";
import { HANDLE_ENDING } from "../../../lib/lens/constants";
import { getMediaURL } from "../../../lib/lens/utils/getMediaURL";
import { PageMetadata } from "../../../types";

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
  const profileImage = profile?.picture ? getMediaURL(profile.picture) : null;
  const coverImage = profile?.coverPicture
    ? getMediaURL(profile.coverPicture)
    : null;

  const metadata: PageMetadata = {
    title,
    description: profile?.bio ?? "",
    image: profileImage,
  };

  return { handle, metadata, profile, coverImage, profileImage };
}
