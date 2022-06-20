import { createContext, useCallback, useEffect, useState } from "react";

import { Publication } from "../../generated/graphql";
import { PlayerLocation } from "../../helpers/app/types";
import {
  SentChatMessage,
  SentIdentity,
  SentWebsocketMessage,
} from "../../helpers/host/types";
import { useLensStore } from "../../helpers/lens/store";

const DEFAULT_HOST = "host.thewired.space";
const WS = process.env.NODE_ENV === "production" ? "wss" : "ws";

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
  space: Publication;
  children: React.ReactNode;
}

export default function SpaceProvider({ space, children }: Props) {
  //if development use localhost
  //else get from space owner profile
  //else use default host
  const host =
    process.env.NODE_ENV === "production"
      ? space.profile.attributes?.find((item) => item.key === "host")?.value ??
        DEFAULT_HOST
      : "localhost:4000";

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
    if (!space || !host) {
      setSocket(undefined);
      return;
    }

    const newSocket = new WebSocket(`${WS}://${host}/${space.id}`);

    newSocket.addEventListener("open", () => {
      console.log("Connected to host server");
      setSocket(newSocket);

      //create analytics event
      //this is used to track the number of people who have connected to the space
      //so we can show popular spaces on the explore page
      //idk if this is a good way to do it but it works for now
      fetch(`/api/space/${space.id}/add-view`);
    });

    newSocket.addEventListener("close", () => {
      console.log("Disconnected from host server");
      setSocket(newSocket);
    });

    return () => {
      newSocket.close();
    };
  }, [space, host]);

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
