import { createContext, useCallback, useEffect, useState } from "react";

import { PlayerLocation } from "../../helpers/app/types";
import {
  SentChatMessage,
  SentIdentity,
  SentWebsocketMessage,
} from "../../helpers/host/types";
import { useLensStore } from "../../helpers/lens/store";

const hostServer = "host.thewired.space";

export interface ISpaceContext {
  socket: WebSocket | undefined;
  sendChatMessage: (message: string) => void;
  sendLocation: (location: PlayerLocation) => void;
}

export const SpaceContext = createContext<ISpaceContext>({
  socket: undefined,
  sendChatMessage: () => {},
  sendLocation: () => {},
});

interface Props {
  spaceId: string | undefined;
  children: React.ReactNode;
}

export default function SpaceProvider({ spaceId, children }: Props) {
  const [socket, setSocket] = useState<WebSocket>();

  const handle = useLensStore((state) => state.handle);

  const sendMessage = useCallback(
    (message: SentWebsocketMessage) => {
      if (!socket) return;
      socket.send(JSON.stringify(message));
    },
    [socket]
  );

  //create the websocket connection
  useEffect(() => {
    if (!spaceId) {
      setSocket(undefined);
      return;
    }

    const newSocket = new WebSocket(`wss://${hostServer}/${spaceId}`);

    newSocket.addEventListener("open", () => {
      console.log("Connected to host server");
      setSocket(newSocket);
    });

    newSocket.addEventListener("close", () => {
      console.log("Disconnected from host server");
      setSocket(newSocket);
    });

    return () => {
      newSocket.close();
    };
  }, [spaceId]);

  //send identity to host
  useEffect(() => {
    if (!socket || !handle) return;

    const identityMessage: SentIdentity = {
      type: "identity",
      data: handle,
    };

    sendMessage(identityMessage);
  }, [socket, handle, sendMessage]);

  //helper functions
  function sendChatMessage(message: string) {
    const websocketMessage: SentChatMessage = {
      type: "chatmessage",
      data: message,
    };

    sendMessage(websocketMessage);
  }

  function sendLocation(location: PlayerLocation) {
    const websocketMessage: SentWebsocketMessage = {
      type: "location",
      data: location,
    };

    sendMessage(websocketMessage);
  }

  return (
    <SpaceContext.Provider value={{ socket, sendChatMessage, sendLocation }}>
      {children}
    </SpaceContext.Provider>
  );
}
