import { Metadata } from "next";
import { notFound } from "next/navigation";

import { fetchSpace } from "../../../src/server/helpers/fetchSpace";
import { fetchSpaceMetadata } from "../../../src/server/helpers/fetchSpaceMetadata";
import RainbowkitWrapper from "../../(navbar)/RainbowkitWrapper";
import SessionProvider from "../../(navbar)/SessionProvider";
import App from "./App";

type Params = { id: string };

export const revalidate = 60;

export async function generateMetadata({ params: { id } }: { params: Params }): Promise<Metadata> {
  const spaceId = parseInt(id);
  const space = await fetchSpace(spaceId);

  if (!space) return {};

  const title = space.metadata?.name ?? `Space ${id}`;
  const description = space.metadata?.description ?? "";

  return {
    title,
    description,
    openGraph: {
      title,
      description,
      creators: space?.profile?.handle?.full ? [space.profile.handle.full] : undefined,
      images: space.metadata?.image ? [{ url: space.metadata.image }] : undefined,
    },
    twitter: {
      title,
      description,
      images: space.metadata?.image ? [space.metadata.image] : undefined,
      card: space.metadata?.image ? "summary_large_image" : "summary",
    },
  };
}

interface Props {
  params: Params;
}

export default async function Play({ params }: Props) {
  const id = parseInt(params.id);
  const metadata = await fetchSpaceMetadata(id);

  if (!metadata) notFound();

  return (
    <SessionProvider>
      <RainbowkitWrapper>
        <App id={id} metadata={metadata} />
      </RainbowkitWrapper>
    </SessionProvider>
  );
}
