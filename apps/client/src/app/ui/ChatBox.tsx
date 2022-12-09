import { useEffect, useRef } from "react";

import { useSubscribeValue } from "../../editor/hooks/useSubscribeValue";
import { useAppStore } from "../store";
import ChatMessage from "./ChatMessage";

export default function ChatBox() {
  const inputRef = useRef<HTMLInputElement>(null);

  const chatBoxFocused = useAppStore((state) => state.chatBoxFocused);
  const chatMessages$ = useAppStore(
    (state) => state.engine?.networkingInterface.chatMessages$
  );

  const chatMessages = useSubscribeValue(chatMessages$);

  useEffect(() => {
    if (!inputRef.current) return;

    if (chatBoxFocused) {
      inputRef.current.focus();
      document.exitPointerLock();
    }
  }, [chatBoxFocused]);

  const focusedClass = chatBoxFocused
    ? "bg-neutral-800/80"
    : "bg-neutral-800/20 hover:bg-neutral-800/30";

  return (
    <div className="flex w-96 flex-col space-y-1">
      <div className="h-full space-y-1">
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
              const value = e.currentTarget.value;
              e.currentTarget.value = "";
              if (!value) return;

              // Send message to server
              useAppStore.getState().engine?.sendChatMessage(value);
            }
          }}
          onFocus={() => useAppStore.setState({ chatBoxFocused: true })}
          onBlur={() => useAppStore.setState({ chatBoxFocused: false })}
          type="text"
          placeholder="Send a message..."
          className={`h-full w-full rounded-md px-4 py-2 text-white outline-none drop-shadow backdrop-blur-2xl transition selection:bg-sky-600 placeholder:text-white/90 placeholder:drop-shadow ${focusedClass}`}
        />
      </div>
    </div>
  );
}
