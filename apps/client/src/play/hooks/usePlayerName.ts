import { useEffect, useState } from "react";

import { trpc } from "../../client/trpc";
import { PlayerName } from "../networking/PlayerName";
import { usePlayStore } from "../store";

export function usePlayerName(playerId: number | null) {
  const players = usePlayStore((state) => state.players);

  const [player, setPlayer] = useState<PlayerName | null>(null);

  const utils = trpc.useContext();

  useEffect(() => {
    if (!players || playerId === null) return;

    const player = players.names.get(playerId) ?? null;
    setPlayer(player);

    return () => {
      setPlayer(null);
    };
  }, [playerId, players, utils]);

  return player;
}
