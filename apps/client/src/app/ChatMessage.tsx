import { useEffect, useState } from "react";

import { useAppStore } from "./store";

interface Props {
  playerId: string;
  message: string;
}

export default function ChatMessage({ playerId, message }: Props) {
  const chatBoxFocused = useAppStore((state) => state.chatBoxFocused);

  const [visible, setVisible] = useState(false);

  useEffect(() => {
    setVisible(true);

    const timeout = setTimeout(() => {
      setVisible(false);
    }, 10000);

    return () => {
      clearTimeout(timeout);
    };
  }, [playerId, message]);

  const visibleClass = chatBoxFocused
    ? "opacity-100 duration-75"
    : visible
    ? "opacity-100"
    : "opacity-0";

  return (
    <div
      className={`w-fit rounded-lg bg-surface/80 px-4 py-1 backdrop-blur-lg transition duration-500 ${visibleClass} `}
    >
      <span className="font-bold">{playerId.substring(0, 6)}</span>: {message}
    </div>
  );
}
