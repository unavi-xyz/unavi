"use client";

import { ValidSpace } from "@/src/server/helpers/validateSpace";
import SpaceCard from "@/src/ui/SpaceCard";

import { useExploreStore } from "./store";

interface Props {
  spaces: ValidSpace[];
}

export default function Spaces({ spaces }: Props) {
  const filter = useExploreStore((state) => state.filter);

  const filteredSpaces = spaces.filter(
    (space) => filter === "" || space.metadata.title.toLowerCase().includes(filter.toLowerCase())
  );

  return (
    <>
      {filteredSpaces.map(({ id, metadata }) => (
        <SpaceCard key={id} id={id} metadata={metadata} />
      ))}
    </>
  );
}
