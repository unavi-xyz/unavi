import { useContext, useEffect, useState } from "react";
import { MultiplayerContext } from "../..";

import OtherPlayer from "./OtherPlayer";

interface Props {
  roomId: string;
}

export function Multiplayer({ roomId }: Props) {
  const { ydoc } = useContext(MultiplayerContext);

  const [players, setPlayers] = useState<string[]>([]);

  useEffect(() => {
    if (!ydoc) return;

    const map = ydoc.getMap("positions");

    function onObserve() {
      const keys = map.keys();
      setPlayers(Array.from(keys)); //! yucky
    }

    map.observe(onObserve);

    return () => {
      map.unobserve(onObserve);
    };
  }, [ydoc]);

  return (
    <group>
      {players.map((id) => {
        return <OtherPlayer key={id} id={id} />;
      })}
    </group>
  );
}
