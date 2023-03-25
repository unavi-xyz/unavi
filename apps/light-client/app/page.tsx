import { Metadata } from "next";
import { notFound } from "next/navigation";

import App from "./App";
import { fetchMetadata } from "./fetchMetadata";

const SPACE_ID = parseInt(process.env.NEXT_PUBLIC_SPACE_ID || "13");

export const metadata: Metadata = {
  title: `Space ${SPACE_ID.toString(16).padStart(4, "0x")} / The Wired`,
};

export default async function Home() {
  const metadata = await fetchMetadata(SPACE_ID);

  if (!metadata) notFound();

  return <App spaceId={SPACE_ID} metadata={metadata} />;
}
