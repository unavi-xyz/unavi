import { fetchLatestSpaces } from "@/src/server/helpers/fetchLatestSpaces";
import CardGrid from "@/src/ui/CardGrid";

import Search from "./Search";
import Spaces from "./Spaces";

export default async function ExploreCard() {
  const spaces = await fetchLatestSpaces(40);

  return (
    <section className="space-y-4">
      <h2 className="text-center text-2xl font-bold">🌏 Explore</h2>

      <div className="flex w-full justify-center">
        <Search />
      </div>

      <CardGrid>
        <Spaces spaces={spaces} />
      </CardGrid>
    </section>
  );
}
