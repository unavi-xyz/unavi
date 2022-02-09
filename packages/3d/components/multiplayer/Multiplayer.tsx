import { CeramicContext } from "ceramic";
import { useContext, useEffect, useState } from "react";
import { MultiplayerContext } from "../..";

import OtherPlayer from "./OtherPlayer";

interface Props {
  roomId: string;
}

export function Multiplayer({ roomId }: Props) {
  const { id } = useContext(CeramicContext);
  const { ydoc, setRoomId } = useContext(MultiplayerContext);

  useEffect(() => {
    if (!setRoomId) return;
    setRoomId(roomId);
  }, [roomId]);

  const [players, setPlayers] = useState<string[]>([]);

  useEffect(() => {
    if (!ydoc || !id) return;

    const map = ydoc.getMap("locations");

    function onObserve() {
      const keys = map.keys();
      const newPlayer = Array.from(keys).find((key) => {
        if (players.includes(key) || key === id) return false;
        return true;
      });

      if (newPlayer) setPlayers(Array.from(new Set([...players, newPlayer])));
    }

    map.observe(onObserve);

    return () => {
      map.unobserve(onObserve);
    };
  }, [ydoc, players, id]);

  return (
    <group>
      {players.map((id) => {
        return <OtherPlayer key={id} id={id} />;
      })}
    </group>
  );
}
