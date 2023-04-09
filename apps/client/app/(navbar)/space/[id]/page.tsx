import { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { notFound, redirect } from "next/navigation";
import { Suspense } from "react";

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

  const { title, description, creator, image } = metadata;

  return {
    title,
    description,
    openGraph: {
      title,
      description,
      creators: creator ? [creator] : undefined,
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
  const creator = metadata.creator.split("/").pop();
  const profileId = creator?.startsWith("0x") && creator.length < 42 ? creator : null;
  const address = creator?.startsWith("0x") && creator.length === 42 ? creator : null;

  const profile = profileId
    ? await fetchProfile(parseInt(profileId))
    : address
    ? await fetchProfileFromAddress(address)
    : null;

  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-8 py-8">
        <div className="flex flex-col space-y-8 md:flex-row md:space-y-0 md:space-x-8">
          <div className="aspect-card h-full w-full rounded-3xl bg-neutral-200">
            <div className="relative h-full w-full object-cover">
              {metadata.image &&
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
                {metadata.title || `Space ${params.id}`}
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
                ) : (
                  <div className="flex justify-center space-x-1 font-bold md:justify-start">
                    <div className="text-neutral-500">By</div>

                    <a href={metadata.creator}>
                      <div className="max-w-xs cursor-pointer overflow-hidden text-ellipsis decoration-2 hover:underline md:max-w-md">
                        {metadata.creator.split("/").pop()}
                      </div>
                    </a>
                  </div>
                )}

                <div className="flex justify-center space-x-1 font-bold md:justify-start">
                  <div className="text-neutral-500">At</div>
                  <div>{metadata.host}</div>
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
