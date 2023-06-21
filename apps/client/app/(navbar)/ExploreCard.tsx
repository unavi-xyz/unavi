import { env } from "@/src/env.mjs";
import { fetchLatestWorlds } from "@/src/server/helpers/fetchLatestWorlds";
import CardGrid from "@/src/ui/CardGrid";

import Search from "./Search";
import Worlds from "./Worlds";

export const runtime = env.PLANETSCALE ? "edge" : "nodejs";

export default async function ExploreCard() {
  const worlds = await fetchLatestWorlds(40);

  return (
    <section className="w-full space-y-4">
      <h2 className="text-center text-3xl font-bold">🌏 Explore</h2>

      <div className="flex justify-center">
        <Search />
      </div>

      <CardGrid>
        <Worlds worlds={worlds} />
      </CardGrid>
    </section>
  );
}
