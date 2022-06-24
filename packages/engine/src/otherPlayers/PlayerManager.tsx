import { useContext, useEffect, useState } from "react";

import { NetworkingContext } from "../networking";
import { RecievedWebsocketMessage } from "../networking/types";
import { OtherPlayer } from "./OtherPlayer";

interface Props {
  animationsUrl: string;
  defaultAvatarUrl: string;
}

export function PlayerManager({ animationsUrl, defaultAvatarUrl }: Props) {
  const [playerIds, setPlayerIds] = useState(new Set<string>());

  const { socket } = useContext(NetworkingContext);

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
    <>
      {Array.from(playerIds).map((playerId) => {
        return (
          <OtherPlayer
            key={playerId}
            id={playerId}
            animationsUrl={animationsUrl}
            defaultAvatarUrl={defaultAvatarUrl}
          />
        );
      })}
    </>
  );
}
