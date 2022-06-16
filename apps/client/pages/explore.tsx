import { NextPageContext } from "next";
import Link from "next/link";

import Carousel from "../src/components/base/Carousel";
import { getNavbarLayout } from "../src/components/layouts/NavbarLayout/NavbarLayout";
import AvatarCard from "../src/components/lens/AvatarCard";
import SpaceCard from "../src/components/lens/SpaceCard";
import MetaTags from "../src/components/ui/MetaTags";
import {
  ExplorePublicationsDocument,
  ExplorePublicationsQuery,
  ExplorePublicationsQueryVariables,
  Post,
  PublicationSortCriteria,
} from "../src/generated/graphql";
import { lensClient } from "../src/helpers/lens/client";
import { AppId } from "../src/helpers/lens/types";

export async function getServerSideProps({ res }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const oneMonthAgo = new Date().getTime() - 30 * 24 * 60 * 60 * 1000;

  const hotSpacesQuery = await lensClient
    .query<ExplorePublicationsQuery, ExplorePublicationsQueryVariables>(
      ExplorePublicationsDocument,
      {
        sources: [AppId.space],
        sortCriteria: PublicationSortCriteria.TopCommented,
        timestamp: oneMonthAgo,
      }
    )
    .toPromise();
  const hotSpaces = hotSpacesQuery.data?.explorePublications.items ?? [];

  const hotAvatarsQuery = await lensClient
    .query<ExplorePublicationsQuery, ExplorePublicationsQueryVariables>(
      ExplorePublicationsDocument,
      {
        sources: [AppId.avatar],
        sortCriteria: PublicationSortCriteria.TopCommented,
        timestamp: oneMonthAgo,
      }
    )
    .toPromise();
  const hotAvatars = hotAvatarsQuery.data?.explorePublications.items ?? [];

  return {
    props: {
      hotSpaces,
      hotAvatars,
    },
  };
}

interface Props {
  hotSpaces: Post[];
  hotAvatars: Post[];
}

export default function Explore({ hotSpaces, hotAvatars }: Props) {
  return (
    <>
      <MetaTags title="Explore" />

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex flex-col items-center justify-center">
            <div className="font-black text-3xl">Explore</div>
          </div>

          {hotSpaces.length > 0 && (
            <Carousel title="🔥 Hot Spaces">
              {hotSpaces.map((space) => (
                <Link key={space.id} href={`/space/${space.id}`} passHref>
                  <a className="h-64 flex-shrink-0">
                    <SpaceCard space={space} />
                  </a>
                </Link>
              ))}
            </Carousel>
          )}

          {hotAvatars.length > 0 && (
            <Carousel title="🔥 Hot Avatars">
              {hotAvatars.map((avatar) => (
                <Link key={avatar.id} href={`/avatar/${avatar.id}`} passHref>
                  <a className="h-96 flex-shrink-0">
                    <AvatarCard avatar={avatar} />
                  </a>
                </Link>
              ))}
            </Carousel>
          )}
        </div>
      </div>
    </>
  );
}

Explore.getLayout = getNavbarLayout;
