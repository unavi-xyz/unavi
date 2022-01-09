import React, { ReactChild, useContext, useEffect, useState } from "react";
import { Room } from "matrix-js-sdk";
import { ClientContext, parseRoomTopic } from "..";

import * as Y from "yjs";
import { WebrtcProvider } from "y-webrtc";

const SIGNALING_SERVERS = [
  "wss://signaling.yjs.dev",
  "wss://y-webrtc-signaling-eu.herokuapp.com",
  "wss://y-webrtc-signaling-us.herokuapp.com",
];

interface ContextInterface {
  room: null | Room;
  ymap: null | Y.Map<any>;
}
const defaultContext: ContextInterface = { room: null, ymap: null };

export const MultiplayerContext = React.createContext(defaultContext);

interface Props {
  children: ReactChild;
}

export function MultiplayerProvider({ children }: Props) {
  const { client } = useContext(ClientContext);

  const [room, setRoom] = useState<ContextInterface["room"]>(null);
  const [ymap, setYmap] = useState<ContextInterface["ymap"]>(null);
  const [ydoc, setYdoc] = useState<null | Y.Doc>(null);

  useEffect(() => {
    if (!client) return;

    const urlParams = new URLSearchParams(window.location.search);
    const roomId = urlParams.get("room");
    if (!roomId) return;

    client
      .joinRoom(roomId)
      .then((res) => {
        setRoom(res);

        client.getStateEvent(roomId, "m.room.topic", "").then((res) => {
          const { topic } = res;
          const worldId = parseRoomTopic(topic);
        });
      })
      .catch((err) => {
        //error joining room
        //probably means the user is not logged in
      });
  }, [client]);

  useEffect(() => {
    if (!room) return;
    if (ydoc) ydoc.destroy();

    const doc = new Y.Doc();

    const opts: any = {
      signaling: SIGNALING_SERVERS,
    };

    new WebrtcProvider(room.roomId, doc, opts);

    doc.on("update", (update: any) => {
      Y.applyUpdate(doc, update);
    });

    const map = doc.getMap("positions");

    setYmap(map);
    setYdoc(doc);
  }, [room]);

  return (
    <MultiplayerContext.Provider value={{ room, ymap }}>
      {children}
    </MultiplayerContext.Provider>
  );
}
