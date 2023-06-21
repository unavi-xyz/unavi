import { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { notFound } from "next/navigation";
import { Suspense } from "react";

import { HOME_SERVER } from "@/src/constants";
import { env } from "@/src/env.mjs";
import {
  fetchUserProfile,
  UserProfile,
} from "@/src/server/helpers/fetchUserProfile";
import { fetchWorld } from "@/src/server/helpers/fetchWorld";
import { isFromCDN } from "@/src/utils/isFromCDN";
import { parseWorldId } from "@/src/utils/parseWorldId";

import PlayerCount from "./PlayerCount";
import Tabs from "./Tabs";

export const runtime = env.PLANETSCALE ? "edge" : "nodejs";
export const revalidate = 60;

type Params = { id: string };

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const id = parseWorldId(params.id);

  const found = await fetchWorld(id);
  if (!found) return {};

  const metadata = found.metadata;

  const displayId = id.value.slice(0, 6);
  const title = metadata.info?.name || `World ${displayId}`;

  const description = metadata.info?.description || "";
  const authors = metadata.info?.authors;
  const image = metadata.info?.image;

  return {
    description,
    openGraph: {
      creators: authors ? authors : undefined,
      description,
      images: image ? [{ url: image }] : undefined,
      title,
    },
    title,
    twitter: {
      card: image ? "summary_large_image" : "summary",
      description,
      images: image ? [image] : undefined,
      title,
    },
  };
}

interface Props {
  params: Params;
}

export default async function World({ params }: Props) {
  const id = parseWorldId(params.id);

  const found = await fetchWorld(id);
  if (!found) notFound();

  const metadata = found.metadata;
  const profiles: UserProfile[] = [];

  if (metadata.info?.authors) {
    await Promise.all(
      metadata.info.authors.map(async (author) => {
        const profile = await fetchUserProfile(author);

        if (!profile) {
          profiles.push({
            home: "",
            metadata: { name: author },
            username: "",
          });
          return;
        }

        profiles.push(profile);
      })
    );
  }

  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-8 py-8">
        <div className="flex flex-col space-y-8 md:flex-row md:space-x-8 md:space-y-0">
          <div className="aspect-card h-full w-full rounded-3xl bg-neutral-200">
            <div className="relative h-full w-full object-cover">
              {metadata.info?.image &&
                (isFromCDN(metadata.info.image) ? (
                  <Image
                    src={metadata.info.image}
                    priority
                    fill
                    sizes="(min-width: 768px) 60vw, 100vw"
                    alt=""
                    className="rounded-3xl object-cover"
                  />
                ) : (
                  <img
                    src={metadata.info.image}
                    sizes="(min-width: 768px) 60vw, 100vw"
                    alt=""
                    className="h-full w-full rounded-3xl object-cover"
                    crossOrigin="anonymous"
                  />
                ))}
            </div>
          </div>

          <div className="flex flex-col justify-between space-y-8 md:w-2/3">
            <div className="space-y-4">
              <div className="text-center text-3xl font-black">
                {metadata.info?.name || `World ${params.id}`}
              </div>

              <div>
                {profiles?.length ? (
                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-neutral-500">By</div>

                    {profiles.map((profile, i) => (
                      <div key={i}>
                        {profile.home === HOME_SERVER ? (
                          <Link href={`/@${profile.username}`}>
                            <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis decoration-2 hover:underline md:max-w-md">
                              {profile.metadata.name || `@${profile.username}`}
                            </div>
                          </Link>
                        ) : (
                          <div className="max-w-xs overflow-hidden text-ellipsis md:max-w-md">
                            {profile.metadata.name}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                ) : null}

                <div className="flex justify-center space-x-1 font-bold md:justify-start">
                  <div className="text-neutral-500">At</div>
                  <div>
                    {metadata.info?.host || env.NEXT_PUBLIC_DEFAULT_HOST}
                  </div>
                </div>

                <PlayerCount
                  uri={found.uri}
                  host={metadata.info?.host || env.NEXT_PUBLIC_DEFAULT_HOST}
                />
              </div>
            </div>

            <Link
              href={`/play?id=${params.id}`}
              className="rounded-full bg-neutral-900 py-3 text-center text-lg font-bold text-white outline-neutral-400 transition hover:scale-105"
            >
              Play
            </Link>
          </div>
        </div>

        <Suspense fallback={null}>
          <Tabs id={{ type: "id", value: params.id }} metadata={metadata} />
        </Suspense>
      </div>
    </div>
  );
}
