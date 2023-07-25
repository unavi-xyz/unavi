import {
  ChatMessage as IChatMessage,
  splitHandle,
  useClientStore,
} from "@unavi/engine";
import { useEffect, useState } from "react";

import { usePlayStore } from "@/app/play/playStore";
import { HOME_SERVER } from "@/src/constants";
import Tooltip from "@/src/ui/Tooltip";

import { usePointerLocked } from "../hooks/usePointerLocked";

interface Props {
  message: IChatMessage;
  alwaysShow?: boolean;
}

export default function ChatMessage({ message, alwaysShow }: Props) {
  const chatBoxFocused = usePlayStore((state) => state.chatBoxFocused);
  const [visible, setVisible] = useState(false);
  const [hidden, setHidden] = useState(false);

  const isPointerLocked = usePointerLocked();

  useEffect(() => {
    setVisible(true);
    setHidden(false);

    // Fade out then hide
    const timeout = setTimeout(() => setVisible(false), 10000);
    const timeout2 = setTimeout(() => setHidden(true), 11000);

    return () => {
      clearTimeout(timeout);
      clearTimeout(timeout2);
    };
  }, [message]);

  const showChat = alwaysShow || chatBoxFocused || !isPointerLocked;

  const fadeClass = showChat || visible ? "opacity-100" : "opacity-0";
  const hiddenClass = showChat || !hidden ? "" : "hidden";

  return (
    <div
      className={`my-0.5 w-fit max-w-full rounded-lg bg-black/50 px-4 py-2 text-white backdrop-blur-lg transition duration-500 ${hiddenClass} ${fadeClass}`}
    >
      {message.type === "player" ? (
        <div className="whitespace-pre-wrap break-words">
          <PlayerName playerId={message.playerId} />:{" "}
          <span className="text-white/90">{message.text}</span>
        </div>
      ) : message.type === "system" ? (
        <span className="text-neutral-400">{message.text}</span>
      ) : null}
    </div>
  );
}

function PlayerName({ playerId }: { playerId: number }) {
  const [name] = useState(useClientStore.getState().getDisplayName(playerId));
  const [handle] = useState(useClientStore.getState().handles.get(playerId));

  if (handle) {
    const { username, server } = splitHandle(handle);

    const fromSameServer = server === HOME_SERVER;

    if (username) {
      return (
        <Tooltip text={handle} side="right" capitalize={false}>
          <span
            className={`${
              fromSameServer ? "font-semibold" : "text-neutral-300"
            }`}
          >
            @{username}
          </span>
        </Tooltip>
      );
    }
  }

  return <span>{name}</span>;
}
