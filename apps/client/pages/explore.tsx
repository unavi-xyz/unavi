import { NextPageContext } from "next";
import Head from "next/head";
import Link from "next/link";

import { getNavbarLayout } from "../src/components/layouts/NavbarLayout/NavbarLayout";
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

  const { data } = await lensClient
    .query<ExplorePublicationsQuery, ExplorePublicationsQueryVariables>(
      ExplorePublicationsDocument,
      {
        sources: [AppId.space],
      }
    )
    .toPromise();

  if (!data) return { props: {} };

  const spaces = data.explorePublications.items;

  return {
    props: {
      spaces,
    },
  };
}

interface Props {
  spaces: Post[] | undefined;
}

export default function Explore({ spaces }: Props) {
  return (
    <>
      <MetaTags title="Explore" />

      <div className="flex justify-center py-8 mx-4">
        <div className="max-w space-y-8">
          <div className="flex flex-col items-center justify-center">
            <div className="font-black text-3xl">Explore</div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            {spaces?.map((space) => (
              <Link key={space.id} href={`/space/${space.id}`}>
                <div>
                  <SpaceCard space={space} />
                </div>
              </Link>
            ))}
          </div>
        </div>
      </div>
    </>
  );
}

Explore.getLayout = getNavbarLayout;
