import { createContext, useEffect, useState } from "react";

import { SentWebsocketMessage } from "./types";

export const NetworkingContext = createContext<{
  socket: WebSocket | undefined;
  sendMessage: (message: SentWebsocketMessage) => void;
}>({
  socket: undefined,
  sendMessage: () => {},
});

interface NetworkingProviderProps {
  spaceId: string;
  host: string;
  children: React.ReactNode;
}

export function NetworkingProvider({
  spaceId,
  host,
  children,
}: NetworkingProviderProps) {
  const [socket, setSocket] = useState<WebSocket>();

  //create the websocket connection
  useEffect(() => {
    if (!spaceId || !host) {
      setSocket(undefined);
      return;
    }

    const newSocket = new WebSocket(`${host}/ws/${spaceId}`);

    newSocket.addEventListener("open", () => {
      console.log("✅ Connected to host server");
      setSocket(newSocket);
    });

    newSocket.addEventListener("close", () => {
      console.log("❌ Disconnected from host server");
      setSocket(newSocket);
    });

    return () => {
      newSocket.close();
    };
  }, [spaceId, host]);

  function sendMessage(message: SentWebsocketMessage) {
    if (socket) {
      socket.send(JSON.stringify(message));
    }
  }

  return (
    <NetworkingContext.Provider
      value={{
        socket,
        sendMessage,
      }}
    >
      {children}
    </NetworkingContext.Provider>
  );
}
