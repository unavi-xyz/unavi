import { useClient } from "@wired-labs/react-client";
import { useEffect, useRef } from "react";

import { usePlayStore } from "@/app/play/[id]/store";

import { usePointerLocked } from "../hooks/usePointerLocked";
import ChatMessage from "./ChatMessage";

interface Props {
  alwaysShow?: boolean;
}

export default function ChatBox({ alwaysShow }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const chatRef = useRef<HTMLDivElement>(null);
  const chatBoxFocused = usePlayStore((state) => state.chatBoxFocused);

  const isPointerLocked = usePointerLocked();
  const { chatMessages, playerId, send } = useClient();

  useEffect(() => {
    if (!inputRef.current) return;

    if (chatBoxFocused) {
      inputRef.current.focus();
      document.exitPointerLock();
    }
  }, [chatBoxFocused]);

  useEffect(() => {
    if (chatBoxFocused || !isPointerLocked) {
      // Scroll to bottom of chat
      if (chatRef.current) chatRef.current.scrollTop = chatRef.current.scrollHeight;
    }
  }, [chatBoxFocused, isPointerLocked]);

  return (
    <div className="space-y-1">
      <div
        ref={chatRef}
        className="flex max-h-[30vh] w-full flex-col-reverse overflow-hidden hover:overflow-y-scroll"
      >
        {chatMessages
          .sort((a, b) => b.timestamp - a.timestamp)
          .map((message) => (
            <ChatMessage key={message.id} message={message} alwaysShow={alwaysShow} />
          ))}
      </div>

      <div className="h-9">
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
          className="h-full w-full rounded-lg bg-neutral-800/40 px-4 text-white outline-none backdrop-blur-xl transition placeholder:text-white/90 hover:bg-neutral-800/50 focus:bg-neutral-800 focus:placeholder:text-white/75"
        />
      </div>
    </div>
  );
}
