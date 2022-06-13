import { NextPageContext } from "next";
import Link from "next/link";

import { getNavbarLayout } from "../src/components/layouts/NavbarLayout/NavbarLayout";
import AvatarCard from "../src/components/lens/AvatarCard";
import SpaceCard from "../src/components/lens/SpaceCard";
import MetaTags from "../src/components/ui/MetaTags";
import {
  ExplorePublicationsDocument,
  ExplorePublicationsQuery,
  ExplorePublicationsQueryVariables,
  Post,
} from "../src/generated/graphql";
import { lensClient } from "../src/helpers/lens/client";
import { AppId } from "../src/helpers/lens/types";

export async function getServerSideProps({ res }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const spacesQuery = await lensClient
    .query<ExplorePublicationsQuery, ExplorePublicationsQueryVariables>(
      ExplorePublicationsDocument,
      {
        sources: [AppId.space],
      }
    )
    .toPromise();

  const spaces = spacesQuery.data?.explorePublications.items ?? [];

  const avatarsQuery = await lensClient
    .query<ExplorePublicationsQuery, ExplorePublicationsQueryVariables>(
      ExplorePublicationsDocument,
      {
        sources: [AppId.avatar],
      }
    )
    .toPromise();

  const avatars = avatarsQuery.data?.explorePublications.items ?? [];

  return {
    props: {
      spaces,
      avatars,
    },
  };
}

interface Props {
  spaces: Post[];
  avatars: Post[];
}

export default function Explore({ spaces, avatars }: Props) {
  return (
    <>
      <MetaTags title="Explore" />

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex flex-col items-center justify-center">
            <div className="font-black text-3xl">Explore</div>
          </div>

          <div className="space-y-2">
            <div className="font-bold text-2xl">Spaces</div>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {spaces.length === 0 ? (
                <div className="text-outline">No spaces found</div>
              ) : (
                spaces.map((space) => (
                  <Link key={space.id} href={`/space/${space.id}`} passHref>
                    <a>
                      <SpaceCard space={space} />
                    </a>
                  </Link>
                ))
              )}
            </div>
          </div>

          <div className="space-y-2">
            <div className="font-bold text-2xl">Avatars</div>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              {avatars.length === 0 ? (
                <div className="text-outline">No avatars found</div>
              ) : (
                avatars.map((avatar) => (
                  <Link key={avatar.id} href={`/avatar/${avatar.id}`} passHref>
                    <a>
                      <AvatarCard avatar={avatar} />
                    </a>
                  </Link>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

Explore.getLayout = getNavbarLayout;
