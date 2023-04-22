"use client";

import { ValidSpace } from "@/src/server/helpers/validateSpaceNFT";
import SpaceCard from "@/src/ui/SpaceCard";

import { useExploreStore } from "./store";

interface Props {
  spaces: ValidSpace[];
}

export default function Spaces({ spaces }: Props) {
  const filter = useExploreStore((state) => state.filter);

  const filteredSpaces = spaces.filter(
    (space) =>
      filter === "" || space.metadata.info?.name?.toLowerCase().includes(filter.toLowerCase())
  );

  return (
    <>
      {filteredSpaces.map(({ id, metadata }) => (
        <SpaceCard
          key={id.value}
          id={id}
          metadata={metadata}
          tokenId={id.type === "tokenId" ? id.value : undefined}
        />
      ))}
    </>
  );
}
