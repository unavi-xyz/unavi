import { useContext, useEffect, useState } from "react";
import { MultiplayerContext } from "../..";

import OtherPlayer from "./OtherPlayer";

interface Props {
  userId: string;
}

export function Multiplayer({ userId }: Props) {
  const { ydoc } = useContext(MultiplayerContext);

  const [players, setPlayers] = useState<string[]>([]);

  useEffect(() => {
    if (!ydoc) return;
    const map = ydoc.getMap("locations");

    function onObserve() {
      const keys = map.keys();
      const newPlayer = Array.from(keys).find((key) => {
        if (players.includes(key) || key === userId) return false;
        return true;
      });

      if (newPlayer) setPlayers(Array.from(new Set([...players, newPlayer])));
    }

    map.observe(onObserve);
    return () => {
      map.unobserve(onObserve);
    };
  }, [ydoc, players]);

  return (
    <group>
      {players.map((id) => {
        return <OtherPlayer key={id} id={id} />;
      })}
    </group>
  );
}
