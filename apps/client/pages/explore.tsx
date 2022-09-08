import Link from "next/link";

import { AppId, Post, PublicationSortCriteria } from "@wired-labs/lens";

import { getNavbarLayout } from "../src/home/layouts/NavbarLayout/NavbarLayout";
import AvatarCard from "../src/home/lens/AvatarCard";
import SpaceCard from "../src/home/lens/SpaceCard";
import { useExploreQuery } from "../src/lib/lens/hooks/useExploreQuery";
import MetaTags from "../src/ui/MetaTags";
import Carousel from "../src/ui/base/Carousel";
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

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex justify-center font-black text-3xl">Explore</div>

          <Carousel
            title="✨ Latest Spaces"
            disableBack={latestSpaces.cursor === 0}
            disableForward={latestSpaces.isLastPage}
            onBack={latestSpaces.back}
            onForward={latestSpaces.next}
            height="h-44"
          >
            {latestSpaces.items.map((space) => (
              <Link key={space.id} href={`/space/${space.id}`} passHref>
                <div
                  className="h-40 transition duration-500"
                  style={{
                    transform: `translate(calc(-${
                      latestSpaces.cursor * spaceLimit
                    }00% + ${
                      spaceLimit > 1 ? Math.min(latestSpaces.cursor, 1) * 15 : 0
                    }%))`,
                  }}
                >
                  <SpaceCard space={space as Post} sizes="227px" animateEnter />
                </div>
              </Link>
            ))}
          </Carousel>

          <Carousel
            title="✨ Latest Avatars"
            disableBack={latestAvatars.cursor === 0}
            disableForward={latestAvatars.isLastPage}
            onBack={latestAvatars.back}
            onForward={latestAvatars.next}
            height="h-72"
          >
            {latestAvatars.items.map((avatar) => (
              <Link key={avatar.id} href={`/avatar/${avatar.id}`} passHref>
                <div
                  className="h-64 transition duration-500"
                  style={{
                    transform: `translate(calc(-${
                      latestAvatars.cursor * avatarLimit
                    }00% + ${
                      avatarLimit > 1
                        ? Math.min(latestAvatars.cursor, 1) * 15
                        : 0
                    }%))`,
                  }}
                >
                  <AvatarCard avatar={avatar} sizes="140px" animateEnter />
                </div>
              </Link>
            ))}
          </Carousel>
        </div>
      </div>
    </>
  );
}

Explore.getLayout = getNavbarLayout;
