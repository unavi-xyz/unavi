import { Triplet } from "@react-three/cannon";
import { CeramicContext } from "ceramic";
import React, {
  Dispatch,
  ReactChild,
  SetStateAction,
  useContext,
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
  publishLocation: ((position: Triplet, rotation: number) => void) | undefined;
}
const defaultContext: ContextInterface = {
  ydoc: undefined,
  setRoomId: undefined,
  publishLocation: undefined,
};

export const MultiplayerContext = React.createContext(defaultContext);

interface Props {
  children: ReactChild | ReactChild[];
}

export function MultiplayerProvider({ children }: Props) {
  const { id } = useContext(CeramicContext);

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

  function publishLocation(position: Triplet, rotation: number) {
    if (!ydoc || !id) return;

    const map = ydoc.getMap("locations");

    const object: LocationObject = { position, rotation };
    map.set(id, object);
  }

  return (
    <MultiplayerContext.Provider value={{ ydoc, setRoomId, publishLocation }}>
      {children}
    </MultiplayerContext.Provider>
  );
}

export type LocationObject = {
  position: Triplet;
  rotation: number;
};
