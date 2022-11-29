import { AppId, Post, PublicationSortCriteria } from "lens";
import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import Link from "next/link";
import { useState } from "react";

import { useExploreQuery } from "../client/lens/hooks/useExploreQuery";
import { useCursor } from "../home/hooks/useCursor";
import { getNavbarLayout } from "../home/layouts/NavbarLayout/NavbarLayout";
import SpaceCard from "../home/lens/SpaceCard";
import SpaceIdCard from "../home/lens/SpaceIdCard";
import MetaTags from "../home/MetaTags";
import { prisma } from "../server/prisma";
import Carousel from "../ui/Carousel";
import { useIsMobile } from "../utils/useIsMobile";

export const getServerSideProps = async ({
  res,
}: GetServerSidePropsContext) => {
  res.setHeader(
    "Cache-Control",
    "public, s-maxage=60, stale-while-revalidate=604800"
  );

  // Get publications
  const publications = await prisma.publication.findMany({
    where: { type: "SPACE" },
    orderBy: { viewCount: "desc" },
    take: 12,
  });

  // Get lens ids
  const hotSpaces: string[] = [];

  publications.forEach((publication) => {
    if (publication.lensId) hotSpaces.push(publication.lensId);
  });

  return {
    props: { hotSpaces },
  };
};

export default function Explore({
  hotSpaces,
}: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const isMobile = useIsMobile();
  const spaceLimit = isMobile ? 1 : 3;
  // const avatarLimit = isMobile ? 1 : 5;

  const [oneMonthAgo] = useState(Date.now() - 1000 * 60 * 60 * 24 * 30);

  const latestSpaces = useExploreQuery(
    spaceLimit,
    [AppId.Space],
    PublicationSortCriteria.Latest,
    oneMonthAgo
  );

  // const latestAvatars = useExploreQuery(
  //   avatarLimit,
  //   [AppId.Avatar],
  //   PublicationSortCriteria.Latest,
  //   oneMonthAgo
  // );

  const [hotSpacesCursor, setHotSpacesCursor] = useState(0);

  const {
    next: hotSpacesNext,
    back: hotSpacesBack,
    isLastPage: isLastHotSpacesPage,
  } = useCursor(
    hotSpaces.length,
    spaceLimit,
    hotSpacesCursor,
    setHotSpacesCursor
  );

  return (
    <>
      <MetaTags title="Explore" />

      <div className="mx-4 flex justify-center py-8">
        <div className="max-w-content space-y-8">
          <div className="flex justify-center text-3xl font-black">Explore</div>

          <Carousel
            title="🔥 Hot Spaces"
            disableBack={hotSpacesCursor === 0}
            disableNext={isLastHotSpacesPage}
            onBack={hotSpacesBack}
            onNext={hotSpacesNext}
            height="h-36 md:h-48"
          >
            {hotSpaces.map((spaceId) => {
              const pageOffset = `-${hotSpacesCursor * spaceLimit}00%`;
              const gapOffset = `-${hotSpacesCursor * spaceLimit * 12}px`;

              return (
                <div
                  key={spaceId}
                  className="transition duration-500"
                  style={{
                    transform: `translate(calc(${pageOffset} + ${gapOffset}))`,
                  }}
                >
                  <Link href={`/space/${spaceId}`}>
                    <div className="h-32 md:h-44">
                      <SpaceIdCard
                        spaceId={spaceId}
                        sizes="293px"
                        animateEnter
                      />
                    </div>
                  </Link>
                </div>
              );
            })}
          </Carousel>

          <Carousel
            title="🌱 Latest Spaces"
            disableBack={latestSpaces.cursor === 0}
            disableNext={latestSpaces.isLastPage}
            onBack={latestSpaces.back}
            onNext={latestSpaces.next}
            height="h-36 md:h-48"
          >
            {latestSpaces.items.map((space) => {
              const pageOffset = `-${latestSpaces.cursor * spaceLimit}00%`;
              const gapOffset = `-${latestSpaces.cursor * spaceLimit * 12}px`;

              return (
                <div
                  key={space.id}
                  className="transition duration-500"
                  style={{
                    transform: `translate(calc(${pageOffset} + ${gapOffset}))`,
                  }}
                >
                  <Link href={`/space/${space.id}`}>
                    <div className="h-32 md:h-44">
                      <SpaceCard
                        space={space as Post}
                        sizes="293px"
                        animateEnter
                      />
                    </div>
                  </Link>
                </div>
              );
            })}
          </Carousel>

          {/* <Carousel
          title="🌱 Latest Avatars"
          disableBack={latestAvatars.cursor === 0}
          disableNext={latestAvatars.isLastPage}
          onBack={latestAvatars.back}
          onNext={latestAvatars.next}
          height="h-72 md:h-80"
        >
          {latestAvatars.items.map((avatar) => {
            const pageOffset = `-${latestAvatars.cursor * avatarLimit}00%`;
            const gapOffset = `-${latestAvatars.cursor * avatarLimit * 12}px`;

            return (
              <Link key={avatar.id} href={`/avatar/${avatar.id}`} passHref>
                <div
                  className="h-64 transition duration-500 md:h-72"
                  style={{
                    transform: `translate(calc(${pageOffset} + ${gapOffset}))`,
                  }}
                >
                  <AvatarCard avatar={avatar} sizes="173px" animateEnter />
                </div>
              </Link>
            );
          })}
        </Carousel> */}
        </div>
      </div>
    </>
  );
}

Explore.getLayout = getNavbarLayout;
