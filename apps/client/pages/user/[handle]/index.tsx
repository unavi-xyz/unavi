import { NextPageContext } from "next";
import Link from "next/link";

import { getNavbarLayout } from "../../../src/components/layouts/NavbarLayout/NavbarLayout";
import ProfileLayout from "../../../src/components/layouts/ProfileLayout/ProfileLayout";
import {
  ProfileLayoutProps,
  getProfileLayoutProps,
} from "../../../src/components/layouts/ProfileLayout/getProfileLayoutProps";
import AvatarCard from "../../../src/components/lens/AvatarCard";
import SpaceCard from "../../../src/components/lens/SpaceCard";
import {
  GetPublicationsDocument,
  GetPublicationsQuery,
  GetPublicationsQueryVariables,
  Post,
} from "../../../src/generated/graphql";
import { lensClient } from "../../../src/helpers/lens/client";
import { HIDDEN_MESSAGE } from "../../../src/helpers/lens/constants";
import { getMediaImageSSR } from "../../../src/helpers/lens/hooks/useMediaImage";
import { AppId } from "../../../src/helpers/lens/types";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const handle = query.handle as string;
  const props = await getProfileLayoutProps(handle);

  if (!props.profile)
    return {
      props: {
        ...props,
        profile: null,
      },
    };

  const publicationsQuery = await lensClient
    .query<GetPublicationsQuery, GetPublicationsQueryVariables>(
      GetPublicationsDocument,
      {
        profileId: props.profile.id,
        sources: [AppId.space, AppId.avatar],
      }
    )
    .toPromise();

  const publications = publicationsQuery.data?.publications.items.map(
    (item) => {
      const postItem = item as Post;
      postItem.metadata.image = getMediaImageSSR(postItem.metadata.media[0]);
      return postItem;
    }
  );

  return {
    props: {
      ...props,
      publications,
    },
  };
}

interface Props extends ProfileLayoutProps {
  publications?: Post[];
}

export default function User({ publications, ...rest }: Props) {
  return (
    <ProfileLayout {...rest}>
      {publications && publications.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-4">
          {publications?.map((publication) => {
            if (publication.appId === AppId.space) {
              if (publication.metadata.content === HIDDEN_MESSAGE) return null;

              return (
                <div key={publication.id} className="w-full md:col-span-2 p-1">
                  <Link href={`/space/${publication.id}`} passHref>
                    <a>
                      <SpaceCard space={publication} />
                    </a>
                  </Link>
                </div>
              );
            } else if (publication.appId === AppId.avatar) {
              if (publication.metadata.content === HIDDEN_MESSAGE) return null;

              return (
                <div key={publication.id} className="w-full p-1">
                  <Link href={`/avatar/${publication.id}`} passHref>
                    <a>
                      <AvatarCard avatar={publication} />
                    </a>
                  </Link>
                </div>
              );
            }
          })}
        </div>
      )}
    </ProfileLayout>
  );
}

User.getLayout = getNavbarLayout;
