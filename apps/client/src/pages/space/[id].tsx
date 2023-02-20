import { createProxySSGHelpers } from "@trpc/react-query/ssg";
import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import Image from "next/image";
import Link from "next/link";

import { useSession } from "../../client/auth/useSession";
import { trpc } from "../../client/trpc";
import { env } from "../../env/client.mjs";
import MetaTags from "../../home/MetaTags";
import { getNavbarLayout } from "../../home/NavbarLayout/NavbarLayout";
import SpaceSettings from "../../home/SpaceSettings";
import { prisma } from "../../server/prisma";
import { appRouter } from "../../server/router/_app";
import ButtonTabs, { TabContent } from "../../ui/ButtonTabs";
import { isFromCDN } from "../../utils/isFromCDN";
import { hexDisplayToNumber, numberToHexDisplay } from "../../utils/numberToHexDisplay";

const host =
  process.env.NODE_ENV === "development" ? "localhost:4000" : env.NEXT_PUBLIC_DEFAULT_HOST;

export const getServerSideProps = async ({ res, query }: GetServerSidePropsContext) => {
  const ONE_MINUTE_IN_SECONDS = 60;
  const ONE_MONTH_IN_SECONDS = 60 * 60 * 24 * 30;

  res.setHeader(
    "Cache-Control",
    `public, max-age=0, s-maxage=${ONE_MINUTE_IN_SECONDS}, stale-while-revalidate=${ONE_MONTH_IN_SECONDS}`
  );

  const hexId = query.id as string;
  const id = hexDisplayToNumber(hexId);

  const ssg = await createProxySSGHelpers({
    router: appRouter,
    ctx: {
      prisma,
      res,
      session: null,
    },
  });

  await ssg.space.byId.prefetch({ id });

  return {
    props: {
      trpcState: ssg.dehydrate(),
      id,
    },
  };
};

export default function Space({ id }: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const { data: space } = trpc.space.byId.useQuery({ id });
  const { data: playerCount } = trpc.public.playerCount.useQuery({ id });

  const metadata = space?.metadata;
  const author = space?.author;

  const { data: session } = useSession();
  const isAuthor = session && session.address === author?.owner;

  const hexId = numberToHexDisplay(id);

  return (
    <>
      <MetaTags
        title={metadata?.name ?? `Space ${hexId}`}
        description={metadata?.description ?? ""}
        image={metadata?.image}
        card="summary_large_image"
      />

      <div className="flex justify-center">
        <div className="max-w-content mx-4 space-y-8 py-8">
          <div className="flex flex-col space-y-8 md:flex-row md:space-y-0 md:space-x-8">
            <div className="aspect-card h-full w-full rounded-2xl bg-neutral-200">
              <div className="relative h-full w-full object-cover">
                {metadata?.image &&
                  (isFromCDN(metadata.image) ? (
                    <Image
                      src={metadata.image}
                      priority
                      fill
                      sizes="40vw"
                      alt=""
                      className="rounded-2xl object-cover"
                    />
                  ) : (
                    <img
                      src={metadata.image}
                      alt=""
                      className="h-full w-full rounded-2xl object-cover"
                      crossOrigin="anonymous"
                    />
                  ))}
              </div>
            </div>

            <div className="flex flex-col justify-between space-y-8 md:w-2/3">
              <div className="space-y-4">
                <div className="text-center text-3xl font-black">{metadata?.name}</div>

                <div className="space-y-1">
                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-neutral-500">By</div>

                    {author && (
                      <Link href={`/user/${numberToHexDisplay(author.id)}`}>
                        <div className="cursor-pointer decoration-2 hover:underline">
                          {author.handle?.string ?? author.owner}
                        </div>
                      </Link>
                    )}
                  </div>

                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-neutral-500">At</div>
                    <div>{host}</div>
                  </div>

                  {playerCount && playerCount > 0 ? (
                    <div className="flex justify-center space-x-1 font-bold md:justify-start">
                      <div>{playerCount}</div>
                      <div className="text-neutral-500">
                        connected player{playerCount === 1 ? null : "s"}
                      </div>
                    </div>
                  ) : null}
                </div>
              </div>

              <Link
                href={`/play/${hexId}`}
                className="rounded-full bg-neutral-900 py-3 text-center text-lg font-bold text-white outline-neutral-400 transition hover:scale-105"
              >
                Join Space
              </Link>
            </div>
          </div>

          <ButtonTabs titles={isAuthor ? ["About", "Settings"] : ["About"]}>
            <TabContent value="About">
              <div className="whitespace-pre-line">{metadata?.description}</div>
            </TabContent>

            {isAuthor && (
              <TabContent value="Settings">
                <SpaceSettings id={id} />
              </TabContent>
            )}
          </ButtonTabs>
        </div>
      </div>
    </>
  );
}

Space.getLayout = getNavbarLayout;
