"use client";

import { FetchedWorld } from "@/src/server/helpers/fetchLatestWorlds";
import WorldCard from "@/src/ui/WorldCard";

import { useExploreStore } from "./exploreStore";

interface Props {
  worlds: FetchedWorld[];
}

export default function Worlds({ worlds }: Props) {
  const filter = useExploreStore((state) => state.filter);

  const filtered = worlds.filter(
    (world) =>
      filter === "" ||
      world.metadata.info?.name?.toLowerCase().includes(filter.toLowerCase())
  );

  return filtered.map(({ id, uri, metadata }) => (
    <WorldCard key={id} id={id} uri={uri} metadata={metadata} />
  ));
}
