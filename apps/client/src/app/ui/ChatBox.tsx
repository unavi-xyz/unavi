import { useEffect, useRef } from "react";

import { useAppStore } from "../../app/store";
import { useIsMobile } from "../../utils/useIsMobile";
import { sendToHost } from "../hooks/useHost";
import ChatMessage from "./ChatMessage";

export default function ChatBox() {
  const inputRef = useRef<HTMLInputElement>(null);

  const chatBoxFocused = useAppStore((state) => state.chatBoxFocused);
  const chatMessages = useAppStore((state) => state.chatMessages);
  const isMobile = useIsMobile();

  useEffect(() => {
    if (!inputRef.current) return;

    if (chatBoxFocused) {
      inputRef.current.focus();
      document.exitPointerLock();
    }
  }, [chatBoxFocused]);

  const focusedClass =
    isMobile || chatBoxFocused
      ? "bg-neutral-800 placeholder:text-white/75"
      : "bg-neutral-800/40 hover:bg-neutral-800/50 placeholder:text-white";

  const scrollClass = isMobile || chatBoxFocused ? "overflow-auto" : "overflow-hidden";

  return (
    <div className="space-y-1">
      <div
        className={`flex w-full flex-col-reverse ${scrollClass}`}
        style={{ maxHeight: "calc(100vh - 140px)" }}
      >
        {chatMessages?.map((message) => (
          <ChatMessage key={message.id} message={message} />
        ))}
      </div>

      <div>
        <input
          ref={inputRef}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();

              const { playerId, players } = useAppStore.getState();
              if (playerId === null || !players) return;

              const text = e.currentTarget.value;
              if (!text) return;

              e.currentTarget.value = "";

              // Send message to server
              sendToHost({ subject: "chat", data: text });
            }
          }}
          onFocus={() => useAppStore.setState({ chatBoxFocused: true })}
          onBlur={() => useAppStore.setState({ chatBoxFocused: false })}
          type="text"
          placeholder="Send a message..."
          className={`h-full w-full rounded-lg px-4 py-2 text-white outline-none drop-shadow backdrop-blur-3xl transition placeholder:drop-shadow ${focusedClass}`}
        />
      </div>
    </div>
  );
}
