import { useClient } from "@wired-labs/react-client";
import { useEffect, useRef } from "react";

import { usePlayStore } from "../../../app/play/[id]/store";
import { useIsMobile } from "../../utils/useIsMobile";
import ChatMessage from "./ChatMessage";

interface Props {
  alwaysShow?: boolean;
}

export default function ChatBox({ alwaysShow }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const chatBoxFocused = usePlayStore((state) => state.chatBoxFocused);

  const isMobile = useIsMobile();
  const { chatMessages, playerId, send } = useClient();

  useEffect(() => {
    if (!inputRef.current) return;

    if (chatBoxFocused) {
      inputRef.current.focus();
      document.exitPointerLock();
    }
  }, [chatBoxFocused]);

  const focusedClass =
    isMobile || chatBoxFocused
      ? "bg-neutral-800 placeholder:text-white/70"
      : "bg-neutral-800/40 hover:bg-neutral-800/50 placeholder:text-white/90";

  const scrollClass = isMobile || chatBoxFocused ? "overflow-auto" : "overflow-hidden";

  return (
    <div className="space-y-1">
      <div
        className={`flex w-full flex-col-reverse ${scrollClass}`}
        style={{ maxHeight: "calc(100vh - 140px)" }}
      >
        {chatMessages
          .sort((a, b) => b.timestamp - a.timestamp)
          .map((message) => (
            <ChatMessage key={message.id} message={message} alwaysShow={alwaysShow} />
          ))}
      </div>

      <div>
        <input
          ref={inputRef}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();

              if (playerId === null) return;

              const text = e.currentTarget.value;
              if (!text) return;

              e.currentTarget.value = "";

              // Send message to server
              send({ subject: "chat", data: text });
            }
          }}
          onFocus={() => usePlayStore.setState({ chatBoxFocused: true })}
          onBlur={() => usePlayStore.setState({ chatBoxFocused: false })}
          type="text"
          placeholder="Send a message..."
          className={`h-full w-full rounded-lg px-4 py-2 text-white outline-none drop-shadow backdrop-blur-2xl transition placeholder:drop-shadow ${focusedClass}`}
        />
      </div>
    </div>
  );
}
