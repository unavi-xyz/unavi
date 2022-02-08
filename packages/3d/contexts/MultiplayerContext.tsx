import React, {
  Dispatch,
  ReactChild,
  SetStateAction,
  useEffect,
  useState,
} from "react";
import { WebrtcProvider } from "y-webrtc";
import * as Y from "yjs";

const SIGNALING_SERVERS = [
  "wss://signaling.yjs.dev",
  "wss://y-webrtc-signaling-eu.herokuapp.com",
  "wss://y-webrtc-signaling-us.herokuapp.com",
];

interface ContextInterface {
  ydoc: Y.Doc | undefined;
  setRoomId: Dispatch<SetStateAction<string | undefined>> | undefined;
}
const defaultContext: ContextInterface = {
  ydoc: undefined,
  setRoomId: undefined,
};

export const MultiplayerContext = React.createContext(defaultContext);

interface Props {
  children: ReactChild;
}

export function MultiplayerProvider({ children }: Props) {
  const [ydoc, setYdoc] = useState<Y.Doc>();
  const [roomId, setRoomId] = useState<string>();

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

  return (
    <MultiplayerContext.Provider value={{ ydoc, setRoomId }}>
      {children}
    </MultiplayerContext.Provider>
  );
}
