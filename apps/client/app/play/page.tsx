import { Metadata } from "next";
import { notFound } from "next/navigation";
import React from "react";
import { z } from "zod";

import {
  httpsSchema,
  idSchema,
  nftSchema,
  processSpaceURI,
} from "@/src/server/helpers/processSpaceURI";
import { readSpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";

import RainbowkitWrapper from "../(navbar)/RainbowkitWrapper";
import SessionProvider from "../(navbar)/SessionProvider";
import App from "./App";

export async function generateMetadata({ searchParams }: Props): Promise<Metadata> {
  const params = searchParamsSchema.safeParse(searchParams);
  if (!params.success) return {};

  const uri = await processSpaceURI(params.data.space);
  if (!uri) return {};

  const metadata = await readSpaceMetadata(uri);
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
  searchParams?: { [key: string]: string | string[] | undefined };
}

export default async function Play({ searchParams }: Props) {
  const params = searchParamsSchema.safeParse(searchParams);
  if (!params.success) return notFound();

  const uri = await processSpaceURI(params.data.space);
  if (!uri) notFound();

  const metadata = await readSpaceMetadata(uri);
  if (!metadata) notFound();

  return (
    <SessionProvider>
      <RainbowkitWrapper>
        <App metadata={metadata} />
      </RainbowkitWrapper>
    </SessionProvider>
  );
}

const searchParamsSchema = z.object({ space: z.union([httpsSchema, nftSchema, idSchema]) });
