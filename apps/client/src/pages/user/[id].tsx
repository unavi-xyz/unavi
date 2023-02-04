import { createProxySSGHelpers } from "@trpc/react-query/ssg";
import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import Link from "next/dist/client/link";
import Head from "next/dist/shared/lib/head";
import Image from "next/image";
import { useRouter } from "next/router";
import { useEffect } from "react";

import { useSession } from "../../client/auth/useSession";
import { trpc } from "../../client/trpc";
import { getNavbarLayout } from "../../home/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../../home/MetaTags";
import ProfilePicture from "../../home/ProfilePicture";
import SpaceCard from "../../home/SpaceCard";
import { prisma } from "../../server/prisma";
import { appRouter } from "../../server/router/_app";
import Spinner from "../../ui/Spinner";
import { isFromCDN } from "../../utils/isFromCDN";
import { hexDisplayToNumber, numberToHexDisplay } from "../../utils/numberToHexDisplay";

export const getServerSideProps = async ({ res, query }: GetServerSidePropsContext) => {
  const ONE_MINUTE_IN_SECONDS = 60;
  const ONE_MONTH_IN_SECONDS = 60 * 60 * 24 * 30;

  res.setHeader(
    "Cache-Control",
    `public, max-age=0, s-maxage=${ONE_MINUTE_IN_SECONDS}, stale-while-revalidate=${ONE_MONTH_IN_SECONDS}`
  );

  const id = query.id as string;
  const isAddress = id.length === 42;

  const ssg = await createProxySSGHelpers({
    router: appRouter,
    ctx: {
      prisma,
      res,
      session: null,
    },
  });

  if (isAddress) {
    await ssg.social.profile.byAddress.prefetch({ address: id });
  } else {
    await ssg.social.profile.byId.prefetch({ id: hexDisplayToNumber(id) });
  }

  return {
    props: {
      trpcState: ssg.dehydrate(),
      id,
    },
  };
};

export default function User({ id }: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const router = useRouter();
  const { data: session, status } = useSession();

  const isAddress = id.length === 42;

  const { data: profileAddress, isLoading: isLoadingAddress } =
    trpc.social.profile.byAddress.useQuery(
      { address: id },
      {
        enabled: isAddress,
        refetchOnWindowFocus: false,
      }
    );

  const { data: profileId, isLoading: isLoadingId } = trpc.social.profile.byId.useQuery(
    { id: hexDisplayToNumber(id) },
    {
      enabled: !isAddress,
      refetchOnWindowFocus: false,
    }
  );

  const profile = isAddress ? profileAddress : profileId;
  const isLoading = isAddress ? isLoadingAddress : isLoadingId;
  const isUser = status === "authenticated" && profile?.owner === session?.address;

  const { data: spaces, isLoading: isLoadingSpaces } = trpc.space.latest.useQuery(
    { owner: profile?.owner },
    {
      enabled: profile?.owner !== undefined,
      refetchOnWindowFocus: false,
    }
  );

  // Force change page on hash change
  useEffect(() => {
    function onHashChange() {
      router.replace(window.location.href);
    }

    window.addEventListener("hashchange", onHashChange);

    return () => {
      window.removeEventListener("hashchange", onHashChange);
    };
  }, [router]);

  if (!isAddress && !isLoading && profile === null)
    return <div className="pt-12 text-center text-lg">User not found.</div>;

  return (
    <>
      <MetaTags
        title={
          profile ? (profile.handle ? profile.handle.string : numberToHexDisplay(profile.id)) : id
        }
        description={profile?.metadata?.description ?? undefined}
        image={profile?.metadata?.image ?? undefined}
      />

      <Head>
        <meta property="og:type" content="profile" />
        <meta property="og:profile:username" content={profile?.handle?.full} />
        <meta property="og:profile:first_name" content={profile?.handle?.string} />
      </Head>

      {isLoading ? (
        <div className="flex justify-center pt-12">
          <Spinner />
        </div>
      ) : (
        <div className="max-w-content mx-auto">
          <div className="h-48 w-full bg-neutral-200 md:h-64 lg:rounded-xl">
            <div className="relative h-full w-full object-cover">
              {profile?.metadata?.animation_url &&
                (isFromCDN(profile.metadata.animation_url) ? (
                  <Image
                    src={profile.metadata.animation_url}
                    priority
                    fill
                    sizes="80vw"
                    alt=""
                    className="h-full w-full object-cover lg:rounded-xl"
                  />
                ) : (
                  <img
                    src={profile.metadata.animation_url}
                    alt=""
                    className="h-full w-full object-cover lg:rounded-xl"
                    crossOrigin="anonymous"
                  />
                ))}
            </div>
          </div>

          <section className="flex justify-center px-4 pb-6 md:px-0">
            <div className="flex w-full flex-col items-center space-y-2">
              <div className="z-10 -mt-16 flex w-32 rounded-full ring-4 ring-white">
                <ProfilePicture
                  src={profile?.metadata?.image}
                  circle
                  uniqueKey={profile?.handle?.full ?? id}
                  size={128}
                />
              </div>

              <div className="flex w-full flex-col items-center">
                {profile?.handle ? (
                  <div>
                    <span className="text-2xl font-black">{profile.handle.string}</span>
                    <span className="text-xl font-bold text-neutral-400">
                      #{profile.handle.id.toString().padStart(4, "0")}
                    </span>
                  </div>
                ) : null}

                <div className="w-full overflow-x-hidden text-ellipsis text-center text-neutral-400">
                  {isAddress ? id : profile?.owner}
                </div>
              </div>

              {profile?.metadata?.description && (
                <div className="w-full whitespace-pre-line text-center">
                  {profile.metadata.description}
                </div>
              )}

              {isUser && (
                <div className="flex w-full justify-center space-x-2">
                  <Link
                    href="/settings"
                    className="rounded-md px-10 py-1.5 font-bold ring-1 ring-neutral-700 transition hover:bg-neutral-200 active:bg-neutral-300"
                  >
                    Edit profile
                  </Link>
                </div>
              )}
            </div>
          </section>

          {isLoadingSpaces ? (
            <div className="flex justify-center pt-12">
              <Spinner />
            </div>
          ) : (
            <div className="grid grid-cols-1 gap-4 px-4 sm:grid-cols-2 md:grid-cols-3">
              {spaces?.map(({ id, metadata }) => {
                return (
                  <Link href={`/space/${numberToHexDisplay(id)}`} key={id}>
                    <SpaceCard id={id} metadata={metadata} animateEnter />
                  </Link>
                );
              })}
            </div>
          )}
        </div>
      )}
    </>
  );
}

User.getLayout = getNavbarLayout;
