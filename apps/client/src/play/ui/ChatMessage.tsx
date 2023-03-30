import { ChatMessage as IChatMessage } from "@wired-labs/react-client";
import { useEffect, useState } from "react";

import { usePlayStore } from "../../../app/play/[id]/store";
import { usePlayerName } from "../hooks/usePlayerName";
import { usePointerLocked } from "../hooks/usePointerLocked";

interface Props {
  message: IChatMessage;
  alwaysShow?: boolean;
}

export default function ChatMessage({ message, alwaysShow }: Props) {
  const chatBoxFocused = usePlayStore((state) => state.chatBoxFocused);
  const [visible, setVisible] = useState(false);
  const [hidden, setHidden] = useState(false);

  const name = usePlayerName(message.playerId);
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
      className={`my-0.5 w-fit max-w-full rounded-lg bg-white px-4 py-1 transition duration-500 ${hiddenClass} ${fadeClass}`}
    >
      {message.type === "player" ? (
        <div className="whitespace-pre-wrap break-words">
          <span className="font-semibold">{name}</span>:{" "}
          <span className="text-neutral-800">{message.text}</span>
        </div>
      ) : message.type === "system" ? (
        <span className="text-neutral-500">
          <span>{name}</span>
          {message.variant === "player_joined"
            ? " joined"
            : message.variant === "player_left"
            ? " left"
            : null}
        </span>
      ) : null}
    </div>
  );
}
