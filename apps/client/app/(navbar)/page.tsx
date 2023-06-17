import { Metadata } from "next";

import AuthProvider from "@/src/client/AuthProvider";

import { metadata as baseMetadata } from "../layout";
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
      <main className="max-w-content mx-4 flex flex-col items-center space-y-8 py-8">
        <h1 className="text-4xl font-black">Welcome to UNAVI</h1>

        <AuthProvider>
          <CreateCard />
        </AuthProvider>

        <ExploreCard />
      </main>
    </div>
  );
}
