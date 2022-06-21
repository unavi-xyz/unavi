import { useContext, useEffect, useState } from "react";

import { usePublishLocation } from "../../helpers/app/hooks/usePublishLocation";
import { RecievedWebsocketMessage } from "../../helpers/host/types";
import OtherPlayer from "./OtherPlayer";
import { SpaceContext } from "./SpaceProvider";

export default function PlayerManager() {
  const [playerIds, setPlayerIds] = useState(new Set<string>());

  const { socket } = useContext(SpaceContext);

  usePublishLocation();

  useEffect(() => {
    if (!socket) return;

    function onMessage(event: MessageEvent) {
      const { type, data } = JSON.parse(event.data) as RecievedWebsocketMessage;

      if (type === "location") {
        //check if new player
        if (!playerIds.has(data.userid)) {
          const newSet = new Set(playerIds);
          newSet.add(data.userid);
          setPlayerIds(newSet);
        }
      }

      if (type === "leave") {
        const newSet = new Set(playerIds);
        newSet.delete(data.userid);
        setPlayerIds(newSet);
      }
    }

    socket.addEventListener("message", onMessage);
    return () => {
      socket.removeEventListener("message", onMessage);
    };
  }, [playerIds, socket]);

  return (
    <group>
      {Array.from(playerIds).map((playerId) => {
        return <OtherPlayer key={playerId} id={playerId} />;
      })}
    </group>
  );
}
