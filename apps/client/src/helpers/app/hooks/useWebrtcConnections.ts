import { useContext, useEffect, useState } from "react";

import { SpaceContext } from "../../../components/app/SpaceProvider";

export function useWebrtcConnections(spaceId: string) {
  const [offers, setOffers] = useState(new Set<string>());
  const [answers, setAnswers] = useState<Record<string, RTCSessionDescription>>(
    {}
  );

  const { socket } = useContext(SpaceContext);

  useEffect(() => {
    if (!socket) return;
    //join the space
    socket.emit("join", spaceId);

    //add existing players
    socket.emit("players", spaceId, (players: string[]) => {
      setOffers((prev) => {
        const newOffers = new Set<string>();
        prev.forEach((item) => newOffers.add(item));
        players.forEach((item) => newOffers.add(item));
        return newOffers;
      });
    });

    //remove player on leave
    socket.on("leave", (id: string) => {
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

    return () => {
      setOffers(new Set());
      setAnswers({});
    };
  }, [socket, spaceId]);

  useEffect(() => {
    if (!socket) return;

    //add player on offer recieve
    function onOffer(id: string, offer: RTCSessionDescription) {
      if (!socket || id === socket.id || offers.has(id)) return;

      setAnswers((prev) => {
        if (prev[id] !== undefined) return prev;
        const clone = { ...prev };
        clone[id] = offer;
        return clone;
      });
    }

    socket.on("offer", onOffer);
    return () => {
      socket.off("offer", onOffer);
    };
  }, [socket, offers]);

  return { offers, answers };
}
