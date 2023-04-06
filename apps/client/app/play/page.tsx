import { Metadata } from "next";
import { cookies } from "next/headers";
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

interface Props {
  searchParams?: { [key: string]: string | string[] | undefined };
}

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

export default async function Play({ searchParams }: Props) {
  // Call cookies() to force this route to be dynamic
  // It should be dynamic because of searchParams, but it's not due to a bug in Next.js
  cookies();

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
