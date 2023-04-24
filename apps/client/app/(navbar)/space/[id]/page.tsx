import { ProfileMetadata } from "@wired-protocol/types";
import { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { notFound, redirect } from "next/navigation";
import { Suspense } from "react";

import { env } from "@/src/env.mjs";
import { fetchDBSpaceURI } from "@/src/server/helpers/fetchDBSpaceURI";
import { fetchProfileFromAddress } from "@/src/server/helpers/fetchProfileFromAddress";
import { fetchSpaceMetadata } from "@/src/server/helpers/fetchSpaceMetadata";
import { isFromCDN } from "@/src/utils/isFromCDN";
import { parseSpaceId } from "@/src/utils/parseSpaceId";
import { toHex } from "@/src/utils/toHex";

import PlayerCount from "./PlayerCount";
import Tabs from "./Tabs";

type Params = { id: string };

export const revalidate = 60;

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const id = parseSpaceId(params.id);

  const space = await fetchSpaceMetadata(id);
  if (!space) return {};

  const metadata = space.metadata;

  const value = id.value;
  const displayId = typeof value === "number" ? toHex(value) : value.slice(0, 6);
  const title = metadata.info?.name || `Space ${displayId}`;

  const description = metadata.info?.description || "";

  const authors = metadata.info?.authors
    ?.map((author) => author.name || author.address)
    .filter(Boolean) as string[] | undefined;

  const image = metadata.info?.image;

  return {
    title,
    description,
    openGraph: {
      title,
      description,
      creators: authors ? authors : undefined,
      images: image ? [{ url: image }] : undefined,
    },
    twitter: {
      title,
      description,
      images: image ? [image] : undefined,
      card: image ? "summary_large_image" : "summary",
    },
  };
}

interface Props {
  params: Params;
}

export default async function Space({ params }: Props) {
  const id = parseSpaceId(params.id);

  // Don't allow uri spaces
  if (id.type === "uri") notFound();

  // If db space has a token, redirect to the token page
  if (id.type === "id") {
    const res = await fetchDBSpaceURI(id.value);
    if (res && res.tokenId !== null) redirect(`/space/${toHex(res.tokenId)}`);
  }

  const space = await fetchSpaceMetadata(id);
  if (!space) notFound();

  const metadata = space.metadata;

  // Fetch author profiles
  const authors = metadata.info?.authors;

  const profiles = authors
    ? await Promise.all(
        authors.map(
          async (author): Promise<{ metadata: ProfileMetadata; id: number | undefined }> => {
            if (author.address) {
              const profile = await fetchProfileFromAddress(author.address);
              return {
                id: profile?.id,
                metadata: {
                  ...profile?.metadata,
                  name: author.name,
                },
              };
            }

            return {
              id: undefined,
              metadata: {
                name: author.name,
              },
            };
          }
        )
      )
    : undefined;

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
                {metadata.info?.name || `Space ${params.id}`}
              </div>

              <div>
                {profiles?.length ? (
                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-neutral-500">By</div>

                    {profiles.map((profile, i) => (
                      <div key={i}>
                        {profile.id !== undefined ? (
                          <Link href={`/user/${toHex(profile.id)}`}>
                            <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis decoration-2 hover:underline md:max-w-md">
                              {profile.metadata.name}
                            </div>
                          </Link>
                        ) : (
                          <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis decoration-2 hover:underline md:max-w-md">
                            {profile.metadata.name}
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                ) : null}

                <div className="flex justify-center space-x-1 font-bold md:justify-start">
                  <div className="text-neutral-500">At</div>
                  <div>{metadata.info?.host || env.NEXT_PUBLIC_DEFAULT_HOST}</div>
                </div>

                <Suspense fallback={null}>
                  {/* @ts-expect-error Server Component */}
                  <PlayerCount uri={metadata.uri} />
                </Suspense>
              </div>
            </div>

            <Link
              href={id.type === "id" ? `/play?id=${id.value}` : `/play?tokenId=${toHex(id.value)}`}
              className="rounded-full bg-neutral-900 py-3 text-center text-lg font-bold text-white outline-neutral-400 transition hover:scale-105"
            >
              Play
            </Link>
          </div>
        </div>

        <Suspense fallback={null}>
          {/* @ts-expect-error Server Component */}
          <Tabs id={id} metadata={metadata} />
        </Suspense>
      </div>
    </div>
  );
}
