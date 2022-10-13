import { InternalChatMessage } from "@wired-labs/engine";
import { useEffect, useState } from "react";

import { usePointerLocked } from "../hooks/usePointerLocked";

interface Props {
  message: InternalChatMessage;
}

export default function ChatMessage({
  message: { message, username, isHandle },
}: Props) {
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

  const visibleClass = !isPointerLocked
    ? "opacity-100"
    : visible
    ? "opacity-100"
    : "opacity-0";

  const boldClass = isHandle ? "font-bold" : null;

  return (
    <div
      className={`w-fit rounded-lg bg-surface px-4 py-1 transition duration-500 ${visibleClass}`}
    >
      <span className={`${boldClass}`}>{username}</span>: {message}
    </div>
  );
}
