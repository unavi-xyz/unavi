"use client";

import { usePlayerCount } from "@/app/api/player-count/helper";

interface Props {
  uri: string;
  host: string;
}

/**
 * Fetches the player count client side and displays it.
 * For use within the space page.
 */
export default function PlayerCount({ uri, host }: Props) {
  const { playerCount } = usePlayerCount({ host, uri });

  if (!playerCount) return null;

  return (
    <div className="flex justify-center space-x-1 font-bold md:justify-start">
      <div>{playerCount}</div>
      <div className="text-neutral-500">connected player{playerCount === 1 ? null : "s"}</div>
    </div>
  );
}
