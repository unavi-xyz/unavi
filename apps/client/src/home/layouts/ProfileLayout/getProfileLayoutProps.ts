import { HANDLE_ENDING, getMediaImageSSR } from "@wired-xr/lens";
import {
  GetProfileDocument,
  GetProfileQuery,
  GetProfileQueryVariables,
  Profile,
} from "@wired-xr/lens/generated/graphql";

import { lensClient } from "../../../lib/lens/client";
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
  const metadata: PageMetadata = {
    title,
    description: profile?.bio ?? "",
    image: getMediaImageSSR(profile?.picture) ?? "",
  };

  const coverImage = getMediaImageSSR(profile?.coverPicture);
  const profileImage = getMediaImageSSR(profile?.picture);

  return { handle, metadata, profile, coverImage, profileImage };
}
