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
}
const defaultContext: ContextInterface = { ydoc: undefined };

export const MultiplayerContext = React.createContext(defaultContext);

interface Props {
  children: ReactChild;
}

export function MultiplayerProvider({ children }: Props) {
  const [ydoc, setYdoc] = useState<Y.Doc>();

  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const roomId = urlParams.get("room");
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
  }, []);

  return (
    <MultiplayerContext.Provider value={{ ydoc }}>
      {children}
    </MultiplayerContext.Provider>
  );
}
