import { useEffect, useState } from "react";

import { useAppStore } from "../store";

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
  alwaysShow?: boolean;
}

export default function ChatMessage({ message, alwaysShow }: Props) {
  const chatBoxFocused = useAppStore((state) => state.chatBoxFocused);
  const [visible, setVisible] = useState(false);
  const [hidden, setHidden] = useState(false);

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

  const fadeClass = alwaysShow || visible || chatBoxFocused ? "opacity-100" : "opacity-0";
  const hiddenClass = alwaysShow || !hidden || chatBoxFocused ? "" : "hidden";

  return (
    <div
      className={`my-0.5 w-fit max-w-full rounded-lg bg-white px-4 py-1 transition duration-500 ${hiddenClass} ${fadeClass}`}
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
