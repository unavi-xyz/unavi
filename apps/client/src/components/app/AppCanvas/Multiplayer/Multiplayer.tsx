import { useContext, useEffect, useState } from "react";
import { useAuth } from "ceramic";

import { appManager, useStore } from "../../helpers/store";
import { SocketContext } from "../../SocketProvider";

import usePublishPosition from "./hooks/usePublishPosition";
import PlayerAnswer from "./PlayerAnswer";
import PlayerOffer from "./PlayerOffer";
import { Identity } from "../../helpers/types";

export default function Multiplayer() {
  const spaceId = useStore((state) => state.spaceId);

  const [offers, setOffers] = useState(new Set<string>());
  const [answers, setAnswers] = useState<Record<string, RTCSessionDescription>>(
    {}
  );

  const { socket } = useContext(SocketContext);

  const { authenticated, viewerId } = useAuth();
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

    //remove player on leave
    socket.on("leave", (id) => {
      setOffers((prev) => {
        const clone = new Set<string>();
        prev.forEach((item) => clone.add(item));
        clone.delete(id);
        return clone;
      });
      setAnswers((prev) => {
        const clone = { ...prev };
        delete clone[id];
        return clone;
      });
    });
  }, [socket, spaceId]);

  useEffect(() => {
    const identity: Identity = { isGuest: !authenticated, did: viewerId };
    appManager.setIdentity(identity);
  }, [authenticated, viewerId]);

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
