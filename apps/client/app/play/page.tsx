import { Metadata } from "next";
import { cookies } from "next/headers";
import { notFound } from "next/navigation";
import React from "react";
import { z } from "zod";

import { fetchSpaceNFTMetadata } from "@/src/server/helpers/fetchSpaceNFTMetadata";
import { httpsSchema, idSchema, nftSchema } from "@/src/server/helpers/processSpaceURI";
import { readSpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";
import { prisma } from "@/src/server/prisma";
import { S3Path } from "@/src/utils/s3Paths";

import RainbowkitWrapper from "../(navbar)/RainbowkitWrapper";
import SessionProvider from "../(navbar)/SessionProvider";
import App from "./App";

interface Props {
  searchParams?: { [key: string]: string | string[] | undefined };
}

export async function generateMetadata({ searchParams }: Props): Promise<Metadata> {
  const params = searchParamsSchema.safeParse(searchParams);
  if (!params.success) return {};

  const metadata = await fetchSpaceMetdata(params.data);
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

export default async function Play({ searchParams }: Props) {
  // Call cookies() to force this route to be dynamic
  // It should be dynamic because of searchParams, but it's not due to a bug in Next.js
  cookies();

  const params = searchParamsSchema.safeParse(searchParams);
  if (!params.success) return notFound();

  const metadata = await fetchSpaceMetdata(params.data);
  if (!metadata) notFound();

  return (
    <SessionProvider>
      <RainbowkitWrapper>
        <App metadata={metadata} />
      </RainbowkitWrapper>
    </SessionProvider>
  );
}

async function fetchSpaceMetdata(params: z.infer<typeof searchParamsSchema>) {
  let uri: string | undefined;

  if ("id" in params) {
    try {
      const space = await prisma.space.findFirst({
        where: { publicId: params.id },
        include: { SpaceModel: true },
      });
      if (!space?.SpaceModel) return null;

      uri = S3Path.space(space.SpaceModel.publicId).model;
    } catch {
      return null;
    }
  } else if ("nft" in params) {
    try {
      // Fetch metadata
      const tokenId = parseInt(params.nft);
      const metadata = await fetchSpaceNFTMetadata(tokenId);

      // No model
      if (!metadata?.animation_url) return null;

      uri = metadata.animation_url;
    } catch {
      return null;
    }
  } else if ("uri" in params) {
    uri = params.uri;
  }

  if (!uri) return null;

  const metadata = await readSpaceMetadata(uri);

  return metadata;
}

const searchParamsSchema = z.union([
  z.object({ id: idSchema }),
  z.object({ nft: nftSchema }),
  z.object({ uri: httpsSchema }),
]);
