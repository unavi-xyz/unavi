import { useEffect, useState } from "react";

import { trpc } from "../../client/trpc";
import { Player } from "../networking/Player";
import { useAppStore } from "../store";

export function usePlayer(playerId: number | null) {
  const players = useAppStore((state) => state.players);

  const [player, setPlayer] = useState<Player | null>(null);

  const utils = trpc.useContext();

  useEffect(() => {
    if (!players || playerId === null) return;

    const player = players.getPlayer(playerId) ?? null;
    setPlayer(player);

    return () => {
      setPlayer(null);
    };
  }, [playerId, players, utils]);

  return player;
}
