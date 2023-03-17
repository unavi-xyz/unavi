import { Metadata } from "next";

import { fetchLatestSpaces } from "../../../src/server/helpers/fetchLatestSpaces";
import CardGrid from "../../../src/ui/CardGrid";
import { metadata as baseMetadata } from "../../layout";
import Search from "./Search";
import Spaces from "./Spaces";

export const revalidate = 60;

const TITLE = "Explore";

export const metadata: Metadata = {
  title: TITLE,
  openGraph: {
    ...baseMetadata.openGraph,
    title: TITLE,
  },
  twitter: {
    ...baseMetadata.twitter,
    title: TITLE,
  },
};

export default async function Explore() {
  const spaces = await fetchLatestSpaces(40);

  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 flex flex-col items-center space-y-8 py-8">
        <div className="text-center text-3xl font-black">Explore</div>

        <Search />

        <CardGrid>
          <Spaces spaces={spaces} />
        </CardGrid>
      </div>
    </div>
  );
}
