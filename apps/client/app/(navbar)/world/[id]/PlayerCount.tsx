"use client";

import { usePlayerCount } from "@/app/api/player-count/helper";

interface Props {
  uri: string;
  host: string;
}

/**
 * Fetches the player count client side and displays it.
 */
export default function PlayerCount({ uri, host }: Props) {
  const { playerCount } = usePlayerCount({ host, uri });

  if (!playerCount) return null;

  return (
    <div className="flex justify-center space-x-1 md:justify-start">
      <div className="font-bold">{playerCount}</div>
      <div className="font-semibold text-neutral-500">
        connected player{playerCount === 1 ? null : "s"}
      </div>
    </div>
  );
}
