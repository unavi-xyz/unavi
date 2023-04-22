import { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { notFound, redirect } from "next/navigation";
import { Suspense } from "react";

import { env } from "@/src/env.mjs";
import { fetchDBSpaceMetadata } from "@/src/server/helpers/fetchDBSpaceMetadata";
import { fetchNFTSpaceMetadata } from "@/src/server/helpers/fetchNFTSpaceMetadata";
import { fetchProfile } from "@/src/server/helpers/fetchProfile";
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

  const metadata = await fetchSpaceMetadata(id);
  if (!metadata) return {};

  const value = id.value;
  const displayId = typeof value === "number" ? toHex(value) : value.slice(0, 6);
  const title = metadata.info?.name || `Space ${displayId}`;

  const description = metadata.info?.description || "";

  const author = metadata.info?.author;

  const image = metadata.info?.image;

  return {
    title,
    description,
    openGraph: {
      title,
      description,
      creators: author ? [author] : undefined,
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

  let metadata;

  if (id.type === "tokenId") {
    metadata = await fetchNFTSpaceMetadata(id.value);
  } else {
    metadata = await fetchDBSpaceMetadata(id.value);

    // If space has a token, redirect to the token page
    if (metadata && metadata.tokenId !== null) redirect(`/space/${toHex(metadata.tokenId)}`);
  }

  if (!metadata) notFound();

  // Fetch creator profile
  const author = metadata.info?.author?.split("/").pop();
  const profileId = author?.startsWith("0x") && author.length < 42 ? author : null;
  const address = author?.startsWith("0x") && author.length === 42 ? author : null;

  const profile = profileId
    ? await fetchProfile(parseInt(profileId))
    : address
    ? await fetchProfileFromAddress(address)
    : null;

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
                {profile ? (
                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-neutral-500">By</div>

                    <Link href={`/user/${toHex(profile.id)}`}>
                      <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis decoration-2 hover:underline md:max-w-md">
                        {profile.handle?.string ? profile.handle.string : profile.owner}
                      </div>
                    </Link>
                  </div>
                ) : metadata.info?.author ? (
                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-neutral-500">By</div>

                    {address ? (
                      <Link href={`/user/${metadata.info.author}`}>
                        <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis decoration-2 hover:underline md:max-w-md">
                          {metadata.info.author}
                        </div>
                      </Link>
                    ) : (
                      <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis decoration-2 hover:underline md:max-w-md">
                        {metadata.info.author}
                      </div>
                    )}
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
