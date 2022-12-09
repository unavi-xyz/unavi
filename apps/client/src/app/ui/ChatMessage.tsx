import { InternalChatMessage } from "engine";
import { useEffect, useState } from "react";

import { usePointerLocked } from "../hooks/usePointerLocked";

interface Props {
  message: InternalChatMessage;
}

export default function ChatMessage({ message }: Props) {
  const isPointerLocked = usePointerLocked();

  const [visible, setVisible] = useState(false);

  useEffect(() => {
    setVisible(true);

    const timeout = setTimeout(() => {
      setVisible(false);
    }, 10000);

    return () => {
      clearTimeout(timeout);
    };
  }, [message]);

  const visibleClass = !isPointerLocked || visible ? "opacity-100" : "opacity-0";

  const boldClass = message.isHandle ? "font-bold" : "font-medium";

  return (
    <div
      className={`w-fit max-w-full rounded-lg bg-white px-4 py-1 transition duration-500 ${visibleClass}`}
    >
      {message.type === "chat" ? (
        <div className="break-words">
          <span className={`${boldClass}`}>{message.username}</span>:{" "}
          <span className="text-neutral-500">{message.message}</span>
        </div>
      ) : message.type === "system" ? (
        <span>
          <span className={`${boldClass}`}>{message.username}</span>
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
