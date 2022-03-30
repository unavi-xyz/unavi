import { useContext, useEffect, useState } from "react";

import { useStore } from "../../helpers/store";
import { SocketContext } from "../../SocketProvider";

import usePublishPosition from "./hooks/usePublishPosition";
import PlayerAnswer from "./PlayerAnswer";
import PlayerOffer from "./PlayerOffer";

export default function Multiplayer() {
  const spaceId = useStore((state) => state.spaceId);

  const [offers, setOffers] = useState(new Set<string>());
  const [answers, setAnswers] = useState<Record<string, RTCSessionDescription>>(
    {}
  );

  const { socket } = useContext(SocketContext);

  usePublishPosition();

  useEffect(() => {
    if (!socket) return;

    socket.emit("join", spaceId);

    //add existing players
    socket.emit("players", spaceId, setOffers);

    //add player on offer recieve
    socket.on("offer", (id, offer) => {
      if (id === socket.id) return;
      setAnswers((prev) => {
        const clone = { ...prev };
        clone[id] = offer;
        return clone;
      });
    });

    // //remove player on leave
    // socket.on("leave", (id) => {
    //   function handleLeave(prev: Set<string>) {
    //     const clone = new Set(prev);
    //     clone.delete(id);
    //     return clone;
    //   }

    //   setOffers(handleLeave);
    //   setAnswers(handleLeave);
    // });
  }, [socket, spaceId]);

  return (
    <group>
      {Array.from(offers).map((id) => {
        return <PlayerOffer key={id} id={id} />;
      })}

      {Object.entries(answers).map(([id, offer]) => {
        return <PlayerAnswer key={id} id={id} offer={offer} />;
      })}
    </group>
  );
}
