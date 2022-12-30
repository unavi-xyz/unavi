import { useEffect, useRef } from "react";

import { useIsMobile } from "../../utils/useIsMobile";
import { sendToHost } from "../hooks/useHost";
import { useAppStore } from "../../app/store";
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
    isMobile || chatBoxFocused ? "bg-neutral-800" : "bg-neutral-800/20 hover:bg-neutral-800/30";

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

              const data = e.currentTarget.value;
              if (!data) return;

              e.currentTarget.value = "";

              // Send message to server
              sendToHost({ subject: "message", data });
            }
          }}
          onFocus={() => useAppStore.setState({ chatBoxFocused: true })}
          onBlur={() => useAppStore.setState({ chatBoxFocused: false })}
          type="text"
          placeholder="Send a message..."
          className={`h-full w-full rounded-md px-4 py-2 text-white outline-none drop-shadow backdrop-blur-2xl transition selection:bg-sky-600 placeholder:text-white/80 placeholder:drop-shadow ${focusedClass}`}
        />
      </div>
    </div>
  );
}
