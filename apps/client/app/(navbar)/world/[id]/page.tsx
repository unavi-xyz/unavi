import { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { notFound } from "next/navigation";
import { Suspense } from "react";

import { env } from "@/src/env.mjs";
import { fetchAuthors } from "@/src/server/helpers/fetchAuthors";
import { fetchWorld } from "@/src/server/helpers/fetchWorld";
import { generateWorldMetadata } from "@/src/server/helpers/generateWorldMetadata";
import { isFromCDN } from "@/src/utils/isFromCDN";
import { parseWorldId } from "@/src/utils/parseWorldId";

import PlayerCount from "./PlayerCount";
import Tabs from "./Tabs";

export const revalidate = 60;

type Params = { id: string };

export function generateMetadata({ params }: Props): Promise<Metadata> {
  return generateWorldMetadata(params.id);
}

interface Props {
  params: Params;
}

export default async function World({ params }: Props) {
  const id = parseWorldId(params.id);

  const found = await fetchWorld(id);
  if (!found?.metadata) notFound();

  const metadata = found.metadata;

  const profiles = await fetchAuthors(metadata);

  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-8 py-8">
        <div className="flex flex-col space-y-8 md:flex-row md:space-x-8 md:space-y-0">
          <div className="aspect-card h-full w-full rounded-3xl bg-neutral-200">
            <div className="relative h-full w-full object-cover">
              {metadata?.image &&
                (isFromCDN(metadata.image) ? (
                  <Image
                    src={metadata.image}
                    priority
                    fill
                    sizes="(min-width: 768px) 60vw, 100vw"
                    alt=""
                    className="rounded-3xl object-cover"
                  />
                ) : (
                  <img
                    src={metadata.image}
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
                {metadata?.title || `World ${params.id}`}
              </div>

              <div>
                {profiles?.length ? (
                  <div className="flex justify-center space-x-1 md:justify-start">
                    <div className="font-semibold text-neutral-500">By</div>

                    {profiles.map((profile, i) => {
                      if (typeof profile === "string") {
                        return (
                          <div
                            key={i}
                            className="max-w-xs overflow-hidden text-ellipsis font-bold md:max-w-md"
                          >
                            {profile}
                          </div>
                        );
                      }

                      switch (profile.type) {
                        case "db": {
                          return (
                            <Link key={i} href={`/@${profile.username}`}>
                              <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis font-bold decoration-2 hover:underline md:max-w-md">
                                {`@${profile.username}`}
                              </div>
                            </Link>
                          );
                        }
                        case "did": {
                          return (
                            <Link key={i} href={`/@${profile.did}`}>
                              <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis font-bold decoration-2 hover:underline md:max-w-md">
                                {`${profile.did}`}
                              </div>
                            </Link>
                          );
                        }
                      }
                    })}
                  </div>
                ) : null}

                <div className="flex justify-center space-x-1 md:justify-start">
                  <div className="font-semibold text-neutral-500">Host</div>
                  <div className="font-bold">
                    {metadata?.host || env.NEXT_PUBLIC_DEFAULT_HOST}
                  </div>
                </div>

                <PlayerCount
                  uri={found.uri}
                  host={metadata?.host || env.NEXT_PUBLIC_DEFAULT_HOST}
                />
              </div>
            </div>

            <Link
              href={`/play?id=${params.id}`}
              className="rounded-full bg-neutral-900 py-3 text-center text-lg font-bold text-white outline-2 outline-offset-4 transition hover:scale-105"
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
