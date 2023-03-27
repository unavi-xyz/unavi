import { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { notFound } from "next/navigation";
import { Suspense } from "react";

import { env } from "../../../../src/env.mjs";
import { fetchSpace } from "../../../../src/server/helpers/fetchSpace";
import { isFromCDN } from "../../../../src/utils/isFromCDN";
import { toHex } from "../../../../src/utils/toHex";
import PlayerCount from "./PlayerCount";
import Tabs from "./Tabs";

const host =
  process.env.NODE_ENV === "development" ? "localhost:4000" : env.NEXT_PUBLIC_DEFAULT_HOST;

type Params = { id: string };

export const revalidate = 60;

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
      card: space.metadata?.image ? "summary_large_image" : "summary",
    },
  };
}

interface Props {
  params: Params;
}

export default async function Space({ params }: Props) {
  const { id } = params;
  const spaceId = parseInt(id);
  const space = await fetchSpace(spaceId);

  if (!space) notFound();

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
                    sizes="(min-width: 768px) 60vw, 100vw"
                    alt=""
                    className="rounded-2xl object-cover"
                  />
                ) : (
                  <img
                    src={space.metadata.image}
                    sizes="(min-width: 768px) 60vw, 100vw"
                    alt=""
                    className="h-full w-full rounded-2xl object-cover"
                    crossOrigin="anonymous"
                  />
                ))}
            </div>
          </div>

          <div className="flex flex-col justify-between space-y-8 md:w-2/3">
            <div className="space-y-4">
              <div className="text-center text-3xl font-black">
                {space.metadata?.name ?? `Space ${id}`}
              </div>

              <div className="space-y-1">
                <div className="flex justify-center space-x-1 font-bold md:justify-start">
                  <div className="text-neutral-500">By</div>

                  {space?.profile ? (
                    <Link href={`/user/${toHex(space.profile.id)}`}>
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

        <Suspense fallback={null}>
          {/* @ts-expect-error Server Component */}
          <Tabs owner={space.owner} params={params} />
        </Suspense>
      </div>
    </div>
  );
}
