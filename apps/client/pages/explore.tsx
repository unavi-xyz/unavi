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
  PaginatedResultInfo,
  Post,
  PublicationSortCriteria,
  PublicationTypes,
} from "../src/generated/graphql";
import { lensClient } from "../src/helpers/lens/client";
import { AppId } from "../src/helpers/lens/types";
import { useQueryPagination } from "../src/helpers/utils/useQueryPagination";

const SPACE_LIMIT = 3;
const AVATAR_LIMIT = 5;

async function fetchHotSpaces(pageInfo?: PaginatedResultInfo) {
  const hotAvatarsQuery = await lensClient
    .query<ExplorePublicationsQuery, ExplorePublicationsQueryVariables>(
      ExplorePublicationsDocument,
      {
        request: {
          sources: [AppId.space],
          sortCriteria: PublicationSortCriteria.TopCommented,
          publicationTypes: [PublicationTypes.Post],
          limit: SPACE_LIMIT,
          cursor: pageInfo?.next,
        },
      }
    )
    .toPromise();

  const explore = hotAvatarsQuery.data?.explorePublications;

  const items = (explore?.items as Post[]) ?? [];
  const info = explore?.pageInfo as PaginatedResultInfo;

  return {
    items,
    info,
  };
}

async function fetchHotAvatars(pageInfo?: PaginatedResultInfo) {
  const hotAvatarsQuery = await lensClient
    .query<ExplorePublicationsQuery, ExplorePublicationsQueryVariables>(
      ExplorePublicationsDocument,
      {
        request: {
          sources: [AppId.avatar],
          sortCriteria: PublicationSortCriteria.TopCommented,
          publicationTypes: [PublicationTypes.Post],
          limit: SPACE_LIMIT,
          cursor: pageInfo?.next,
        },
      }
    )
    .toPromise();

  const explore = hotAvatarsQuery.data?.explorePublications;

  const items = (explore?.items as Post[]) ?? [];
  const info = explore?.pageInfo as PaginatedResultInfo;

  return {
    items,
    info,
  };
}

export async function getServerSideProps({ res }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const oneMonthAgo = new Date().getTime() - 30 * 24 * 60 * 60 * 1000;

  const pageInfo: PaginatedResultInfo = {
    totalCount: -1,
    next: JSON.stringify({ timestamp: oneMonthAgo }),
  };

  const hotSpaces = await fetchHotSpaces(pageInfo);
  const hotAvatars = await fetchHotAvatars(pageInfo);

  const props: Props = {
    initialHotSpaces: hotSpaces.items,
    initialHotSpacesInfo: hotSpaces.info,
    initialHotAvatars: hotAvatars.items,
    initialHotAvatarsInfo: hotAvatars.info,
  };

  return {
    props,
  };
}

interface Props {
  initialHotSpaces: Post[];
  initialHotSpacesInfo: PaginatedResultInfo;
  initialHotAvatars: Post[];
  initialHotAvatarsInfo: PaginatedResultInfo;
}

export default function Explore({
  initialHotSpaces,
  initialHotSpacesInfo,
  initialHotAvatars,
  initialHotAvatarsInfo,
}: Props) {
  const hotSpaces = useQueryPagination({
    pageSize: SPACE_LIMIT,
    initialCache: initialHotSpaces,
    initialPageInfo: initialHotSpacesInfo,
    fetchNextPage: fetchHotSpaces,
  });

  const hotAvatars = useQueryPagination({
    pageSize: AVATAR_LIMIT,
    initialCache: initialHotAvatars,
    initialPageInfo: initialHotAvatarsInfo,
    fetchNextPage: fetchHotAvatars,
  });

  return (
    <>
      <MetaTags title="Explore" />

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex flex-col items-center justify-center">
            <div className="font-black text-3xl">Explore</div>
          </div>

          {hotSpaces.window.length > 0 && (
            <Carousel
              title="🔥 Hot Spaces"
              back={!hotSpaces.disableBack}
              forward={!hotSpaces.disableNext}
              onBack={hotSpaces.back}
              onForward={hotSpaces.next}
            >
              {hotSpaces.window.map((space) => (
                <Link key={space.id} href={`/space/${space.id}`} passHref>
                  <a className="w-1/3">
                    <SpaceCard space={space} />
                  </a>
                </Link>
              ))}
            </Carousel>
          )}

          {hotAvatars.window.length > 0 && (
            <Carousel
              title="🔥 Hot Avatars"
              back={!hotAvatars.disableBack}
              forward={!hotAvatars.disableNext}
              onBack={hotAvatars.back}
              onForward={hotAvatars.next}
            >
              {hotAvatars.window.map((avatar) => (
                <Link key={avatar.id} href={`/avatar/${avatar.id}`} passHref>
                  <a className="w-1/5">
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
