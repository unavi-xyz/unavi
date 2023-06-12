import { Metadata } from "next";

import { fetchLatestSpaces } from "@/src/server/helpers/fetchLatestSpaces";
import CardGrid from "@/src/ui/CardGrid";

import { metadata as baseMetadata } from "../layout";
import Search from "./Search";
import Spaces from "./Spaces";

export const revalidate = 60;

const title = "Home";

export const metadata: Metadata = {
  openGraph: {
    ...baseMetadata.openGraph,
    title,
  },
  title,
  twitter: {
    ...baseMetadata.twitter,
    title,
  },
};

export default async function Home() {
  const spaces = await fetchLatestSpaces(40);

  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 flex flex-col items-center py-8">
        <div className="text-center text-3xl font-black">Welcome to UNAVI</div>

        <div className="flex w-full justify-center pb-8 pt-6">
          <Search />
        </div>

        <CardGrid>
          <Spaces spaces={spaces} />
        </CardGrid>
      </div>
    </div>
  );
}
