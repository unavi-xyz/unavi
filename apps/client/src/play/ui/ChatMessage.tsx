import {
  ChatMessage as IChatMessage,
  useClientStore,
} from "@unavi/react-client";
import { useEffect, useState } from "react";

import { usePlayStore } from "@/app/play/playStore";
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
        <span className="text-white/70">{message.text}</span>
      ) : null}
    </div>
  );
}

function PlayerName({ playerId }: { playerId: number }) {
  const [name] = useState(useClientStore.getState().getDisplayName(playerId));

  const isHandle = name.startsWith("@");

  if (isHandle) {
    const parts = name.split(":");
    const username = parts[0]?.slice(1);
    const home = parts[1];

    if (username && home) {
      return (
        <Tooltip text={name} side="right" capitalize={false}>
          <span className="font-bold">@{username}</span>
        </Tooltip>
      );
    }
  }

  return <span>{name}</span>;
}
