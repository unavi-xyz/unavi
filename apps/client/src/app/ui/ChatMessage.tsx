import { useEffect, useState } from "react";

import { usePointerLocked } from "../hooks/usePointerLocked";

export type ChatMessage =
  | {
      type: "chat";
      id: string;
      timestamp: number;
      playerId: number;
      username: string;
      message: string;
    }
  | {
      type: "system";
      variant: "player_joined" | "player_left";
      id: string;
      timestamp: number;
      playerId: number;
      username: string;
    };

interface Props {
  message: ChatMessage;
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

  return (
    <div
      className={`my-0.5 w-fit max-w-full rounded-lg bg-white px-4 py-1 transition duration-500 ${visibleClass}`}
    >
      {message.type === "chat" ? (
        <div className="break-words">
          <span>{message.username}</span>: <span>{message.message}</span>
        </div>
      ) : message.type === "system" ? (
        <span className="text-neutral-500">
          <span>{message.username}</span>
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
