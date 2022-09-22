import { AppId, Post, PublicationSortCriteria } from "@wired-labs/lens";
import Link from "next/link";

import { getNavbarLayout } from "../src/home/layouts/NavbarLayout/NavbarLayout";
import AvatarCard from "../src/home/lens/AvatarCard";
import SpaceCard from "../src/home/lens/SpaceCard";
import { useExploreQuery } from "../src/lib/lens/hooks/useExploreQuery";
import Carousel from "../src/ui/base/Carousel";
import MetaTags from "../src/ui/MetaTags";
import { useIsMobile } from "../src/utils/useIsMobile";

export default function Explore() {
  const isMobile = useIsMobile();
  const spaceLimit = isMobile ? 1 : 3;
  const avatarLimit = isMobile ? 1 : 5;

  const latestSpaces = useExploreQuery(
    spaceLimit,
    [AppId.Space],
    PublicationSortCriteria.Latest
  );

  const latestAvatars = useExploreQuery(
    avatarLimit,
    [AppId.Avatar],
    PublicationSortCriteria.Latest
  );

  return (
    <>
      <MetaTags title="Explore" />

      <div className="mx-4 flex justify-center py-8">
        <div className="max-w-content space-y-8">
          <div className="flex justify-center text-3xl font-black">Explore</div>

          <Carousel
            title="✨ Latest Spaces"
            disableBack={latestSpaces.cursor === 0}
            disableForward={latestSpaces.isLastPage}
            onBack={latestSpaces.back}
            onForward={latestSpaces.next}
            height="h-36 md:h-48"
          >
            {latestSpaces.items.map((space) => {
              const pageOffset = `-${latestSpaces.cursor * spaceLimit}00%`;
              const gapOffset = `-${latestSpaces.cursor * spaceLimit * 12}px`;

              return (
                <Link key={space.id} href={`/space/${space.id}`} passHref>
                  <div
                    className="h-32 transition duration-500 md:h-44"
                    style={{
                      transform: `translate(calc(${pageOffset} + ${gapOffset}))`,
                    }}
                  >
                    <SpaceCard
                      space={space as Post}
                      sizes="293px"
                      animateEnter
                    />
                  </div>
                </Link>
              );
            })}
          </Carousel>

          <Carousel
            title="✨ Latest Avatars"
            disableBack={latestAvatars.cursor === 0}
            disableForward={latestAvatars.isLastPage}
            onBack={latestAvatars.back}
            onForward={latestAvatars.next}
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
                    <AvatarCard avatar={avatar} sizes="140px" animateEnter />
                  </div>
                </Link>
              );
            })}
          </Carousel>
        </div>
      </div>
    </>
  );
}

Explore.getLayout = getNavbarLayout;
