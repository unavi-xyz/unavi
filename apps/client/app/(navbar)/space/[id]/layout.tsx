import { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { notFound } from "next/navigation";
import { Suspense } from "react";

import { env } from "../../../../src/env/client.mjs";
import { fetchSpace } from "../../../../src/server/helpers/fetchSpace";
import { getServerSession } from "../../../../src/server/helpers/getServerSession";
import NavigationTab from "../../../../src/ui/NavigationTab";
import { isFromCDN } from "../../../../src/utils/isFromCDN";
import { numberToHexDisplay } from "../../../../src/utils/numberToHexDisplay";
import PlayerCount from "./PlayerCount";

const host =
  process.env.NODE_ENV === "development" ? "localhost:4000" : env.NEXT_PUBLIC_DEFAULT_HOST;

type Params = { id: string };

export async function generateMetadata({ params: { id } }: { params: Params }): Promise<Metadata> {
  const spaceId = parseInt(id);
  const space = await fetchSpace(spaceId);

  if (!space) return {};

  const title = space.metadata?.name ?? `Space ${id}`;
  const description = space.metadata?.description ?? "";

  return {
    title,
    description,
    openGraph: {
      title,
      description,
      creators: space?.profile?.handle?.full ? [space.profile.handle.full] : undefined,
      images: space.metadata?.image ? [{ url: space.metadata.image }] : undefined,
    },
    twitter: {
      title,
      description,
      images: space.metadata?.image ? [space.metadata.image] : undefined,
    },
  };
}

interface Props {
  children: React.ReactNode;
  params: Params;
}

export default async function Space({ children, params: { id } }: Props) {
  const spaceId = parseInt(id);
  const [session, space] = await Promise.all([getServerSession(), fetchSpace(spaceId)]);

  if (!space) notFound();

  const isOwner = session?.address === space.owner;

  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-8 py-8">
        <div className="flex flex-col space-y-8 md:flex-row md:space-y-0 md:space-x-8">
          <div className="aspect-card h-full w-full rounded-2xl bg-neutral-200">
            <div className="relative h-full w-full object-cover">
              {space.metadata?.image &&
                (isFromCDN(space.metadata.image) ? (
                  <Image
                    src={space.metadata.image}
                    priority
                    fill
                    sizes="40vw"
                    alt=""
                    className="rounded-2xl object-cover"
                  />
                ) : (
                  <img
                    src={space.metadata.image}
                    alt=""
                    className="h-full w-full rounded-2xl object-cover"
                    crossOrigin="anonymous"
                  />
                ))}
            </div>
          </div>

          <div className="flex flex-col justify-between space-y-8 md:w-2/3">
            <div className="space-y-4">
              <div className="text-center text-3xl font-black">{space.metadata?.name}</div>

              <div className="space-y-1">
                <div className="flex justify-center space-x-1 font-bold md:justify-start">
                  <div className="text-neutral-500">By</div>

                  {space?.profile ? (
                    <Link href={`/user/${numberToHexDisplay(space.profile.id)}`}>
                      <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis decoration-2 hover:underline md:max-w-md">
                        {space.profile.handle?.string ?? space.owner}
                      </div>
                    </Link>
                  ) : space?.owner ? (
                    <Link href={`/user/${space.owner}`}>
                      <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis decoration-2 hover:underline md:max-w-md">
                        {space.owner}
                      </div>
                    </Link>
                  ) : null}
                </div>

                <div className="flex justify-center space-x-1 font-bold md:justify-start">
                  <div className="text-neutral-500">At</div>
                  <div>{host}</div>
                </div>

                <Suspense fallback={null}>
                  {/* @ts-expect-error Server Component */}
                  <PlayerCount id={spaceId} />
                </Suspense>
              </div>
            </div>

            <Link
              href={`/play/${id}`}
              className="rounded-full bg-neutral-900 py-3 text-center text-lg font-bold text-white outline-neutral-400 transition hover:scale-105"
            >
              Enter Space
            </Link>
          </div>
        </div>

        {isOwner && (
          <div className="flex space-x-4">
            <NavigationTab
              text="About"
              href={`/space/${id}`}
              exact
              className="w-full rounded-lg text-lg"
            />
            <NavigationTab
              text="Settings"
              href={`/space/${id}/settings`}
              exact
              className="w-full rounded-lg text-lg"
            />
          </div>
        )}

        <div>{children}</div>
      </div>
    </div>
  );
}
