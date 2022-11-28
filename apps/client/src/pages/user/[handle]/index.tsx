import {
  AppId,
  GetPublicationsDocument,
  GetPublicationsQuery,
  GetPublicationsQueryVariables,
  Post,
  PublicationTypes,
} from "lens";
import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import Link from "next/link";

import { HIDDEN_MESSAGE } from "../../../client/lens/constants";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import { getProfileLayoutProps } from "../../../home/layouts/ProfileLayout/getProfileLayoutProps";
import ProfileLayout from "../../../home/layouts/ProfileLayout/ProfileLayout";
import AvatarCard from "../../../home/lens/AvatarCard";
import SpaceCard from "../../../home/lens/SpaceCard";
import { lensClient } from "../../../server/lens";
import { getMediaURL } from "../../../utils/getMediaURL";

export const getServerSideProps = async ({
  res,
  query,
}: GetServerSidePropsContext) => {
  res.setHeader(
    "Cache-Control",
    "public, s-maxage=30, stale-while-revalidate=3600"
  );

  const handle = query.handle as string;
  const props = await getProfileLayoutProps(handle);

  if (!props.profile) {
    return {
      props: {
        ...props,
        profile: null,
        publications: null,
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

  const publications =
    publicationsQuery.data?.publications.items.map((item) => {
      const postItem = item as Post;
      postItem.metadata.image = getMediaURL(postItem.metadata.media[0]);
      return postItem;
    }) ?? null;

  return {
    props: {
      ...props,
      publications,
    },
  };
};

export default function User({
  publications,
  ...rest
}: InferGetServerSidePropsType<typeof getServerSideProps>) {
  return (
    <ProfileLayout {...rest}>
      {publications && publications.length > 0 && (
        <div className="grid grid-cols-1 gap-4 pb-4 md:grid-cols-4">
          {publications.map((publication) => {
            if (publication.appId === AppId.Space) {
              if (publication.metadata.content === HIDDEN_MESSAGE) return null;

              return (
                <div key={publication.id} className="w-full md:col-span-2">
                  <Link
                    href={`/space/${publication.id}`}
                    passHref
                    legacyBehavior
                  >
                    <div>
                      <SpaceCard space={publication} sizes="49vw" />
                    </div>
                  </Link>
                </div>
              );
            } else if (publication.appId === AppId.Avatar) {
              if (publication.metadata.content === HIDDEN_MESSAGE) return null;

              return (
                <div key={publication.id} className="w-full">
                  <Link
                    href={`/avatar/${publication.id}`}
                    passHref
                    legacyBehavior
                  >
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
