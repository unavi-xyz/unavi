import { Metadata } from "next";
import { notFound } from "next/navigation";
import { z } from "zod";

import { WORLD_ID_LENGTH } from "@/src/server/db/constants";
import {
  ClientIdentityProfile,
  fetchProfile,
} from "@/src/server/helpers/fetchProfile";
import { fetchWorld } from "@/src/server/helpers/fetchWorld";
import { generateWorldMetadata } from "@/src/server/helpers/generateWorldMetadata";
import { parseIdentity } from "@/src/utils/parseIdentity";

import App from "./App";
import { WorldUriId } from "./types";

export const dynamic = "force-dynamic";

interface Props {
  searchParams?: { [key: string]: string | string[] | undefined };
}

export async function generateMetadata({
  searchParams,
}: Props): Promise<Metadata> {
  const params = searchParamsSchema.safeParse(searchParams);
  if (!params.success) return {};

  const id = parseParams(params.data);

  return await generateWorldMetadata(id.value);
}

export default async function Play({ searchParams }: Props) {
  const params = searchParamsSchema.safeParse(searchParams);
  if (!params.success) return notFound();

  const id = parseParams(params.data);
  const world = await fetchWorld(id);

  if (!world?.metadata) notFound();

  const authors: Array<string | ClientIdentityProfile> = await Promise.all(
    world.metadata.authors.map(async (author) => {
      const identity = parseIdentity(author);
      const profile = await fetchProfile(identity);

      if (profile) {
        return {
          ...profile,
          id: undefined,
        };
      }

      return author;
    })
  );

  return (
    <App id={id} uri={world.uri} metadata={world.metadata} authors={authors} />
  );
}

function parseParams(params: Params): WorldUriId {
  if ("id" in params) return { type: "id", value: params.id };
  else return { type: "uri", value: params.uri };
}

const httpsSchema = z.string().refine((param) => param.startsWith("https://"));

const idSchema = z.string().refine((param) => {
  return param.length === WORLD_ID_LENGTH;
});

const searchParamsSchema = z.union([
  z.object({ id: idSchema }),
  z.object({ uri: httpsSchema }),
]);

type Params = z.infer<typeof searchParamsSchema>;
