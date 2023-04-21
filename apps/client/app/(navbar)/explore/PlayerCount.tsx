"use client";

import { MdPeople } from "react-icons/md";

import { usePlayerCount } from "@/app/api/player-count/helper";
import { SpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";

interface Props {
  metadata: SpaceMetadata;
}

export default function PlayerCount({ metadata }: Props) {
  const { playerCount } = usePlayerCount({ host: metadata.host, uri: metadata.uri });

  if (!playerCount) return null;

  return (
    <div className="absolute flex h-full w-full items-start justify-end pr-4 pt-3.5 tracking-wide">
      <div className="flex items-center space-x-2 rounded-full bg-black/50 px-3.5 py-0.5 text-white backdrop-blur">
        <MdPeople className="text-xl" />
        <div className="text-lg font-bold">{playerCount}</div>
      </div>
    </div>
  );
}
