import produce from "immer";
import { createContext, useEffect, useState } from "react";
import { Socket, io } from "socket.io-client";

import { createMessage } from "../../helpers/app/message";
import { useAppStore } from "../../helpers/app/store";
import { Channels } from "../../helpers/app/types";

const hostServer =
  process.env.NODE_ENV === "development"
    ? "localhost:8080"
    : "https://signaling.thewired.space";

type ChannelStore = Record<keyof Channels, RTCDataChannel[]>;

const channelStore: ChannelStore = {
  identity: [],
  location: [],
  message: [],
};

interface ISpaceContext {
  socket: Socket | undefined;
  channels: ChannelStore | undefined;
  connections: RTCPeerConnection[] | undefined;
  addChannel: (name: keyof Channels, channel: RTCDataChannel) => void;
  addConnection: (connection: RTCPeerConnection) => void;
  removeConnection: (connection: RTCPeerConnection) => void;
  createAnswer: (connection: RTCPeerConnection, id: string) => Promise<void>;
  createOffer: (connection: RTCPeerConnection, id: string) => Promise<void>;
  publishAll: (type: keyof Channels, data: any) => void;
}

const defaultContext: ISpaceContext = {
  socket: undefined,
  channels: undefined,
  connections: undefined,
  addChannel: () => {},
  addConnection: () => {},
  removeConnection: () => {},
  createAnswer: () => Promise.resolve(),
  createOffer: () => Promise.resolve(),
  publishAll: () => {},
};

export const SpaceContext = createContext(defaultContext);

interface Props {
  spaceId: string;
  children: React.ReactNode;
}

export default function SpaceProvider({ spaceId, children }: Props) {
  const [socket, setSocket] = useState<Socket>();
  const [channels, setChannels] = useState<ChannelStore>(channelStore);
  const [connections, setConnections] = useState<RTCPeerConnection[]>([]);

  useEffect(() => {
    const newSocket = io(hostServer);
    setSocket(newSocket);

    return () => {
      newSocket.disconnect();
    };
  }, []);

  async function createAnswer(connection: RTCPeerConnection, id: string) {
    if (!socket) return;
    const answer = await connection.createAnswer();
    connection.setLocalDescription(answer).catch(() => {});
    socket.emit("answer", id, answer);
  }

  async function createOffer(connection: RTCPeerConnection, id: string) {
    if (!socket) return;
    const offer = await connection.createOffer();
    connection.setLocalDescription(offer).catch(() => {});
    socket.emit("offer", id, offer);
  }

  function addChannel(type: keyof Channels, channel: RTCDataChannel) {
    setChannels(
      produce((draft) => {
        draft[type].push(channel);

        if (type === "identity") {
          // const identity = useAppStore.getState().identity;
          // publish(channel, identity);
        }
      })
    );
  }

  function addConnection(connection: RTCPeerConnection) {
    setConnections(
      produce((draft) => {
        draft.push(connection);
      })
    );
  }

  function removeConnection(connection: RTCPeerConnection) {
    setConnections(
      produce((draft) => {
        draft = draft.filter((c) => c !== connection);
      })
    );
  }

  function publish(channel: RTCDataChannel, data: any) {
    if (channel.readyState === "open") {
      channel.send(JSON.stringify(data));
      return;
    }

    channel.onopen = () => channel.send(JSON.stringify(data));
  }

  function publishAll(type: keyof Channels, data: any) {
    channels[type].forEach((channel) => {
      publish(channel, data);
    });

    if (type === "message") {
      const localMessage = createMessage(data, "");
      useAppStore.getState().addMessage(localMessage);
    }
  }

  return (
    <SpaceContext.Provider
      value={{
        socket,
        channels,
        connections,
        addChannel,
        addConnection,
        removeConnection,
        createAnswer,
        createOffer,
        publishAll,
      }}
    >
      {children}
    </SpaceContext.Provider>
  );
}
