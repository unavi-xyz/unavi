import React, { ReactChild, useEffect, useState } from "react";
import * as Y from "yjs";
import { WebrtcProvider } from "y-webrtc";

const SIGNALING_SERVERS = [
  "wss://signaling.yjs.dev",
  "wss://y-webrtc-signaling-eu.herokuapp.com",
  "wss://y-webrtc-signaling-us.herokuapp.com",
];

interface ContextInterface {
  ymap: Y.Map<any> | undefined;
}
const defaultContext: ContextInterface = { ymap: undefined };

export const MultiplayerContext = React.createContext(defaultContext);

interface Props {
  children: ReactChild;
}

export function MultiplayerProvider({ children }: Props) {
  const [ymap, setYmap] = useState<Y.Map<any>>();
  const [ydoc, setYdoc] = useState<Y.Doc>();

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const roomId = urlParams.get("room");
    if (!roomId) return;

    ydoc?.destroy();

    const doc = new Y.Doc();

    const opts: any = {
      signaling: SIGNALING_SERVERS,
    };

    // new WebrtcProvider(roomId, doc, opts);

    doc.on("update", (update: any) => {
      Y.applyUpdate(doc, update);
    });

    const map = doc.getMap("positions");

    setYmap(map);
    setYdoc(doc);
  }, []);

  return (
    <MultiplayerContext.Provider value={{ ymap }}>
      {children}
    </MultiplayerContext.Provider>
  );
}
