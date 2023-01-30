import { useEffect, useState } from "react";

import { usePointerLocked } from "../hooks/usePointerLocked";

export type ChatMessage =
  | {
      type: "chat";
      id: string;
      timestamp: number;
      playerId: number;
      displayName: string;
      text: string;
    }
  | {
      type: "system";
      variant: "player_joined" | "player_left";
      id: string;
      timestamp: number;
      playerId: number;
      displayName: string;
    };

interface Props {
  message: ChatMessage;
}

export default function ChatMessage({ message }: Props) {
  const isPointerLocked = usePointerLocked();
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    setVisible(true);

    // Hide after 10 seconds
    const timeout = setTimeout(() => setVisible(false), 10000);

    return () => clearTimeout(timeout);
  }, [message]);

  const visibleClass = !isPointerLocked || visible ? "opacity-100" : "opacity-0";

  return (
    <div
      className={`my-0.5 w-fit max-w-full rounded-lg bg-white px-4 py-1 transition duration-500 ${visibleClass}`}
    >
      {message.type === "chat" ? (
        <div className="whitespace-pre break-words">
          <span className="font-bold">{message.displayName}</span>: <span>{message.text}</span>
        </div>
      ) : message.type === "system" ? (
        <span className="text-neutral-500">
          <span>{message.displayName}</span>
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
