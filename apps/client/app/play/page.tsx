import { Metadata } from "next";
import { cookies } from "next/headers";
import { notFound } from "next/navigation";
import { z } from "zod";

import AuthProvider from "@/src/client/AuthProvider";
import { fetchSpaceMetadata } from "@/src/server/helpers/fetchSpaceMetadata";
import { toHex } from "@/src/utils/toHex";

import RainbowkitWrapper from "../(navbar)/RainbowkitWrapper";
import { SPACE_ID_LENGTH } from "../api/spaces/constants";
import App from "./App";
import { SpaceUriId } from "./types";

interface Props {
  searchParams?: { [key: string]: string | string[] | undefined };
}

export async function generateMetadata({
  searchParams,
}: Props): Promise<Metadata> {
  const params = searchParamsSchema.safeParse(searchParams);
  if (!params.success) return {};

  const id = parseParams(params.data);
  const space = await fetchSpaceMetadata(id);
  if (!space) return {};

  const metadata = space.metadata;

  const value = id.value;
  const displayId = typeof value === "number" ? toHex(value) : value;
  const title = metadata.info?.name || `Space ${displayId}`;

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
      creator: authors ? authors[0] : undefined,
      description,
      images: image ? [image] : undefined,
      title,
    },
  };
}

export default async function Play({ searchParams }: Props) {
  // Call cookies() to force this route to be dynamic
  // It should be dynamic because of searchParams, but it's not due to a bug in Next.js
  cookies();

  const params = searchParamsSchema.safeParse(searchParams);
  if (!params.success) return notFound();

  const id = parseParams(params.data);
  const space = await fetchSpaceMetadata(id);

  if (!space) notFound();

  return (
    <AuthProvider>
      <RainbowkitWrapper>
        <App id={id} uri={space.uri} metadata={space.metadata} />
      </RainbowkitWrapper>
    </AuthProvider>
  );
}

function parseParams(params: Params): SpaceUriId {
  if ("id" in params) return { type: "id", value: params.id };
  else if ("tokenId" in params)
    return { type: "tokenId", value: parseInt(params.tokenId) };
  else return { type: "uri", value: params.uri };
}

const httpsSchema = z.string().refine((param) => param.startsWith("https://"));

const idSchema = z.string().refine((param) => {
  return param.length === SPACE_ID_LENGTH;
});

const tokenIdSchema = z.string().refine((param) => {
  return param.startsWith("0x");
});

const searchParamsSchema = z.union([
  z.object({ id: idSchema }),
  z.object({ tokenId: tokenIdSchema }),
  z.object({ uri: httpsSchema }),
]);

type Params = z.infer<typeof searchParamsSchema>;
