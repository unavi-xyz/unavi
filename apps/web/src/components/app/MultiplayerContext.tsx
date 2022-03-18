import { createContext, useEffect, useState } from "react";
import { Triplet } from "@react-three/cannon";
import { WebrtcProvider } from "y-webrtc";
import { applyUpdate, Doc } from "yjs";
import { useAuth } from "ceramic";

const SIGNALING_SERVERS = [
  "wss://signaling.yjs.dev",
  "wss://y-webrtc-signaling-eu.herokuapp.com",
  "wss://y-webrtc-signaling-us.herokuapp.com",
];

const defaultValue = {
  joinSpace: (spaceId: string) => {},
  publishLocation: (position: Triplet, rotation: number) => {},
  ydoc: new Doc(),
};

export const MultiplayerContext = createContext(defaultValue);

export default function MultiplayerProvider({ children }) {
  const [ydoc, setYdoc] = useState(new Doc());
  const [spaceName, setSpaceName] = useState("");

  const { viewerId } = useAuth();

  useEffect(() => {
    const doc = new Doc();

    new WebrtcProvider(spaceName, doc, {
      signaling: SIGNALING_SERVERS,
    } as any);

    doc.on("update", (update: any) => {
      applyUpdate(doc, update);
    });

    setYdoc(doc);

    return () => {
      doc?.destroy();
    };
  }, [spaceName, setYdoc]);

  function joinSpace(spaceId: string) {
    if (spaceId === spaceName) return;
    setSpaceName(spaceId);
  }

  function publishLocation(position: Triplet, rotation: number) {
    if (!ydoc || !viewerId) return;
    const map = ydoc.getMap("locations");
    const object = { position, rotation };
    map.set(viewerId, object);
  }

  return (
    <MultiplayerContext.Provider value={{ joinSpace, publishLocation, ydoc }}>
      {children}
    </MultiplayerContext.Provider>
  );
}
