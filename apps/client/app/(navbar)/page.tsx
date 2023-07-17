import { Metadata } from "next";

import { metadata as baseMetadata } from "../layout";
import CommunityCard from "./CommunityCard";
import CreateCard from "./CreateCard";
import ExploreCard from "./ExploreCard";

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
  return (
    <div className="flex justify-center">
      <main className="max-w-content mx-4 flex flex-col items-center space-y-12 py-8">
        <h1 className="text-4xl font-black">Welcome to UNAVI</h1>

        <div className="flex w-full flex-col space-y-8 md:flex-row md:space-x-8 md:space-y-0">
          <CommunityCard />
          <CreateCard />
        </div>

        <ExploreCard />
      </main>
    </div>
  );
}
