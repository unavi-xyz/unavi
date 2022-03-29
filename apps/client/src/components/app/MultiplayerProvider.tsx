import {
  createContext,
  Dispatch,
  SetStateAction,
  useEffect,
  useState,
} from "react";
import { nanoid } from "nanoid";
import { WebrtcProvider } from "y-webrtc";
import { applyUpdate, Doc } from "yjs";
import { useAuth } from "ceramic";

import { useStore } from "./helpers/store";
import { emptyUser, Message, YLocation, YUser } from "./helpers/types";

const SIGNALING_SERVERS = [
  "wss://signaling.yjs.dev",
  "wss://y-webrtc-signaling-eu.herokuapp.com",
  "wss://y-webrtc-signaling-us.herokuapp.com",
];

interface IMultiplayerContext {
  ydoc: Doc;
  username: string;
  setRoomId: Dispatch<SetStateAction<string>>;
  publishMessage: (text: string) => void;
  publishLocation: (location: YLocation) => void;
  getMessages: () => void;
  getLocations: () => { [key: string]: YLocation };
}

const defaultValue: IMultiplayerContext = {
  ydoc: undefined,
  username: undefined,
  setRoomId: undefined,
  publishMessage: undefined,
  publishLocation: undefined,
  getMessages: undefined,
  getLocations: undefined,
};

export const MultiplayerContext = createContext(defaultValue);

export default function MultiplayerProvider({ children }) {
  const [ydoc, setYdoc] = useState<Doc>();
  const [roomId, setRoomId] = useState<string>();
  const [username, setUsername] = useState<string>();

  const { viewerId } = useAuth();

  useEffect(() => {
    const doc = new Doc();

    new WebrtcProvider(roomId, doc, {
      signaling: SIGNALING_SERVERS,
    } as any);

    doc.on("update", (update: any) => {
      applyUpdate(doc, update);
    });

    setYdoc(doc);

    return () => {
      doc.destroy();
    };
  }, [roomId]);

  useEffect(() => {
    if (viewerId) {
      setUsername(viewerId);
    } else {
      if (!username) {
        setUsername(nanoid());
      }
    }
  }, [username, viewerId]);

  function getMap() {
    return ydoc.getMap("users");
  }

  function getUser() {
    const map = getMap();
    const user = (map.get(username) as YUser) ?? { ...emptyUser, id: username };
    return user;
  }

  function setUser(value: YUser) {
    const map = getMap();
    const user = getUser();

    map.set(user.id, value);
  }

  function publishLocation(location: YLocation) {
    const user = getUser();
    user.location = location;
    setUser(user);
  }

  function getLocations() {
    const map = getMap();

    const currentLocations: { [key: string]: YLocation } = {};
    map.forEach((user: YUser) => {
      if (user.id === username) return;
      currentLocations[user.id] = user.location;
    });

    return currentLocations;
  }

  function getMessages() {
    const map = getMap();
    const messages = useStore.getState().messages;

    function isNewMessage(message: Message) {
      let wasFound = false;

      messages.forEach(({ text, time, username }) => {
        if (
          text === message.text &&
          time === message.time &&
          username === message.username
        ) {
          wasFound = true;
        }
      });

      return !wasFound;
    }

    const newMessages: Message[] = [];

    map.forEach((user: YUser, key) => {
      const formatted = user.messages
        .map(({ text, time }) => {
          const object: Message = { id: nanoid(), username: key, text, time };
          if (isNewMessage(object)) return object;
        })
        .filter((item) => item !== undefined);

      newMessages.push(...formatted);
    });

    if (newMessages.length > 0) {
      const combined = [...messages, ...newMessages];
      const sorted = combined.sort((a, b) => a.time - b.time);

      useStore.setState({ messages: sorted });
    }
  }

  function sendChatMessage(text: string) {
    const map = getMap();
    const user = getUser();

    const time = Date.now();
    const object = { text, time };
    user.messages.push(object);

    setUser(user);
  }

  return (
    <MultiplayerContext.Provider
      value={{
        ydoc,
        username,
        setRoomId,
        publishMessage: sendChatMessage,
        getMessages,
        publishLocation,
        getLocations,
      }}
    >
      {children}
    </MultiplayerContext.Provider>
  );
}
