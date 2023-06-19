"use client";

import { ValidWorld } from "@/src/server/helpers/validateWorldNFT";
import SpaceCard from "@/src/ui/SpaceCard";

import { useExploreStore } from "./exploreStore";

interface Props {
  worlds: ValidWorld[];
}

export default function Worlds({ worlds }: Props) {
  const filter = useExploreStore((state) => state.filter);

  const filtered = worlds.filter(
    (world) =>
      filter === "" ||
      world.metadata.info?.name?.toLowerCase().includes(filter.toLowerCase())
  );

  return (
    <>
      {filtered.map(({ id, uri, metadata }) => (
        <SpaceCard
          key={id.value}
          id={id}
          uri={uri}
          metadata={metadata}
          tokenId={id.type === "tokenId" ? id.value : undefined}
        />
      ))}
    </>
  );
}
