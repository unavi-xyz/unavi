import { NextPageContext } from "next";
import Link from "next/link";

import { getNavbarLayout } from "../../../src/components/layouts/NavbarLayout/NavbarLayout";
import ProfileLayout from "../../../src/components/layouts/ProfileLayout/ProfileLayout";
import {
  ProfileLayoutProps,
  getProfileLayoutProps,
} from "../../../src/components/layouts/ProfileLayout/getProfileLayoutProps";
import AvatarCard from "../../../src/components/lens/AvatarCard";
import {
  GetPublicationsDocument,
  GetPublicationsQuery,
  GetPublicationsQueryVariables,
  Post,
} from "../../../src/generated/graphql";
import { lensClient } from "../../../src/helpers/lens/client";
import { AppId } from "../../../src/helpers/lens/types";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const handle = query.handle as string;
  const props = await getProfileLayoutProps(handle);

  if (!props.profile) return { props };

  const avatarsQuery = await lensClient
    .query<GetPublicationsQuery, GetPublicationsQueryVariables>(
      GetPublicationsDocument,
      {
        profileId: props.profile.id,
        sources: [AppId.avatar],
      }
    )
    .toPromise();

  const avatars = avatarsQuery.data?.publications.items;

  return {
    props: {
      ...props,
      avatars,
    },
  };
}

interface Props extends ProfileLayoutProps {
  avatars?: Post[];
}

export default function Avatars({ avatars, ...rest }: Props) {
  return (
    <ProfileLayout {...rest}>
      {avatars && avatars.length > 0 && (
        <div className="grid md:grid-cols-3 gap-2">
          {avatars?.map((avatar) => (
            <div key={avatar.id}>
              <Link href={`/avatar/${avatar.id}`} passHref>
                <a>
                  <AvatarCard avatar={avatar} />
                </a>
              </Link>
            </div>
          ))}
        </div>
      )}
    </ProfileLayout>
  );
}

Avatars.getLayout = getNavbarLayout;
