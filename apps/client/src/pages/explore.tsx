import { createProxySSGHelpers } from "@trpc/react-query/ssg";
import Link from "next/link";
import { GetServerSidePropsContext } from "next/types";

import { trpc } from "../client/trpc";
import MetaTags from "../home/MetaTags";
import { getNavbarLayout } from "../home/NavbarLayout/NavbarLayout";
import SpaceCard from "../home/SpaceCard";
import { prisma } from "../server/prisma";
import { appRouter } from "../server/router/_app";
import { numberToHexDisplay } from "../utils/numberToHexDisplay";

export const getServerSideProps = async ({ res }: GetServerSidePropsContext) => {
  const ONE_MINUTE_IN_SECONDS = 60;
  const ONE_MONTH_IN_SECONDS = 60 * 60 * 24 * 30;

  res.setHeader(
    "Cache-Control",
    `public, max-age=0, s-maxage=${ONE_MINUTE_IN_SECONDS}, stale-while-revalidate=${ONE_MONTH_IN_SECONDS}`
  );

  const ssg = await createProxySSGHelpers({
    router: appRouter,
    ctx: {
      prisma,
      res,
      session: null,
    },
  });

  await ssg.space.latest.prefetch({ limit: 25 });

  return {
    props: {
      trpcState: ssg.dehydrate(),
    },
  };
};

export default function Explore() {
  const { data: spaces } = trpc.space.latest.useQuery(
    { limit: 25 },
    {
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
    }
  );

  return (
    <>
      <MetaTags title="Explore" />

      <div className="mx-4 flex justify-center py-8">
        <div className="max-w-content space-y-8">
          <div className="flex justify-center text-3xl font-black">Explore</div>

          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
            {spaces?.map(({ id, metadata }) => {
              return (
                <Link href={`/space/${numberToHexDisplay(id)}`} key={id}>
                  <SpaceCard id={id} metadata={metadata} animateEnter />
                </Link>
              );
            })}
          </div>
        </div>
      </div>
    </>
  );
}

Explore.getLayout = getNavbarLayout;
