import {
  GetProfileDocument,
  GetProfileQuery,
  GetProfileQueryVariables,
  Profile,
} from "@wired-labs/lens";

import { HANDLE_ENDING } from "../../../lib/lens/constants";
import { lensClient } from "../../../server/lens";
import { PageMetadata } from "../../../ui/MetaTags";
import { getMediaURL } from "../../../utils/getMediaURL";

export interface ProfileLayoutProps {
  handle: string;
  metadata: PageMetadata;
  profile: Profile | null;
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
  const profileImage = getMediaURL(profile?.picture);
  const coverImage = getMediaURL(profile?.coverPicture);

  const metadata: PageMetadata = {
    title,
    description: profile?.bio ?? "",
    image: profileImage,
  };

  return { handle, metadata, profile, coverImage, profileImage };
}
