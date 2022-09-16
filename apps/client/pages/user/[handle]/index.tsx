import {
  AppId,
  GetPublicationsDocument,
  GetPublicationsQuery,
  GetPublicationsQueryVariables,
  Post,
  PublicationTypes,
} from "@wired-labs/lens";
import { NextPageContext } from "next";
import Link from "next/link";

import { getNavbarLayout } from "../../../src/home/layouts/NavbarLayout/NavbarLayout";
import {
  getProfileLayoutProps,
  ProfileLayoutProps,
} from "../../../src/home/layouts/ProfileLayout/getProfileLayoutProps";
import ProfileLayout from "../../../src/home/layouts/ProfileLayout/ProfileLayout";
import AvatarCard from "../../../src/home/lens/AvatarCard";
import SpaceCard from "../../../src/home/lens/SpaceCard";
import { lensClient } from "../../../src/lib/lens/client";
import { HIDDEN_MESSAGE } from "../../../src/lib/lens/constants";
import { getMediaURL } from "../../../src/utils/getMediaURL";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const handle = query.handle as string;
  const props = await getProfileLayoutProps(handle);

  if (!props.profile) {
    return {
      props: {
        ...props,
        profile: null,
      },
    };
  }

  const publicationsQuery = await lensClient
    .query<GetPublicationsQuery, GetPublicationsQueryVariables>(
      GetPublicationsDocument,
      {
        request: {
          profileId: props.profile.id,
          sources: [AppId.Space, AppId.Avatar],
          publicationTypes: [PublicationTypes.Post],
        },
      }
    )
    .toPromise();

  const publications = publicationsQuery.data?.publications.items.map(
    (item) => {
      const postItem = item as Post;
      postItem.metadata.image = getMediaURL(postItem.metadata.media[0]);
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
            if (publication.appId === AppId.Space) {
              if (publication.metadata.content === HIDDEN_MESSAGE) return null;

              return (
                <div key={publication.id} className="w-full p-1 md:col-span-2">
                  <Link href={`/space/${publication.id}`} passHref>
                    <a>
                      <SpaceCard space={publication} sizes="49vw" />
                    </a>
                  </Link>
                </div>
              );
            } else if (publication.appId === AppId.Avatar) {
              if (publication.metadata.content === HIDDEN_MESSAGE) return null;

              return (
                <div key={publication.id} className="w-full p-1">
                  <Link href={`/avatar/${publication.id}`} passHref>
                    <div>
                      <AvatarCard avatar={publication} sizes="24vw" />
                    </div>
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
