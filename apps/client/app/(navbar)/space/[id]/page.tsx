import { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { notFound } from "next/navigation";
import { Suspense } from "react";

import { fetchProfile } from "@/src/server/helpers/fetchProfile";
import { processSpaceURI } from "@/src/server/helpers/processSpaceURI";
import { readSpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";
import { prisma } from "@/src/server/prisma";
import { isFromCDN } from "@/src/utils/isFromCDN";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";
import { toHex } from "@/src/utils/toHex";

import PlayerCount from "./PlayerCount";
import Tabs from "./Tabs";

type Params = { id: string };

export const revalidate = 60;

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const uri = `nft://${params.id}`;

  const spaceURI = await processSpaceURI(uri);
  if (!spaceURI) notFound();

  const metadata = await readSpaceMetadata(spaceURI);
  if (!metadata) notFound();

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
  const space = await prisma.space.findFirst({
    where: { publicId: params.id },
    select: { SpaceModel: true, owner: true },
  });
  if (!space || !space.SpaceModel) notFound();

  const modelURI = cdnURL(S3Path.space(space.SpaceModel.publicId).model);
  const metadata = await readSpaceMetadata(modelURI);
  if (!metadata) notFound();

  const profileId = metadata.creator.split("/").pop();
  const profile = profileId ? await fetchProfile(parseInt(profileId)) : null;

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

              <div className="space-y-1">
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
                  <PlayerCount uri={modelURI} />
                </Suspense>
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
          {/* @ts-expect-error Server Component */}
          <Tabs id={params.id} owner={space.owner} description={metadata.description} />
        </Suspense>
      </div>
    </div>
  );
}
