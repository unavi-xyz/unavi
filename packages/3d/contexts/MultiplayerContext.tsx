import { Triplet } from "@react-three/cannon";
import React, { ReactChild, useEffect, useState } from "react";
import { WebrtcProvider } from "y-webrtc";
import * as Y from "yjs";

const SIGNALING_SERVERS = [
  "wss://signaling.yjs.dev",
  "wss://y-webrtc-signaling-eu.herokuapp.com",
  "wss://y-webrtc-signaling-us.herokuapp.com",
];

interface ContextInterface {
  ydoc: Y.Doc | undefined;
  publishLocation: ((position: Triplet, rotation: number) => void) | undefined;
}
const defaultContext: ContextInterface = {
  ydoc: undefined,
  publishLocation: undefined,
};

export const MultiplayerContext = React.createContext(defaultContext);

interface Props {
  roomId: string;
  userId: string;
  children: ReactChild | ReactChild[];
}

export function MultiplayerProvider({ roomId, userId, children }: Props) {
  const [ydoc, setYdoc] = useState<Y.Doc>();

  useEffect(() => {
    if (!roomId) return;
    const doc = new Y.Doc();
    const opts: any = {
      signaling: SIGNALING_SERVERS,
    };
    new WebrtcProvider(roomId, doc as any, opts);

    doc.on("update", (update: any) => {
      Y.applyUpdate(doc, update);
    });

    setYdoc(doc);
    return () => {
      doc.destroy();
    };
  }, [roomId]);

  function publishLocation(position: Triplet, rotation: number) {
    if (!ydoc) return;
    const map = ydoc.getMap("locations");
    const object: LocationObject = { position, rotation };
    map.set(userId, object);
  }

  return (
    <MultiplayerContext.Provider value={{ ydoc, publishLocation }}>
      {children}
    </MultiplayerContext.Provider>
  );
}

export type LocationObject = {
  position: Triplet;
  rotation: number;
};
