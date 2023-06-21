import { useEffect, useRef } from "react";

import { usePlayStore } from "@/app/play/store";

import { usePointerLocked } from "../hooks/usePointerLocked";

interface Props {
  alwaysShow?: boolean;
}

export default function ChatBox({ alwaysShow }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const chatRef = useRef<HTMLDivElement>(null);
  const chatBoxFocused = usePlayStore((state) => state.chatBoxFocused);

  const isPointerLocked = usePointerLocked();

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
      if (chatRef.current)
        chatRef.current.scrollTop = chatRef.current.scrollHeight;
    }
  }, [chatBoxFocused, isPointerLocked]);

  return (
    <div className="space-y-1">
      <div
        ref={chatRef}
        className="flex max-h-[30vh] w-full flex-col-reverse overflow-hidden hover:overflow-y-scroll"
      >
        {/* {chatMessages
          .sort((a, b) => b.timestamp - a.timestamp)
          .map((message) => (
            <ChatMessage
              key={message.id}
              message={message}
              alwaysShow={alwaysShow}
            />
          ))} */}
      </div>

      <div className="h-12">
        <input
          ref={inputRef}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();

              // if (playerId === null) return;

              const text = e.currentTarget.value;
              if (!text) return;

              e.currentTarget.value = "";

              // Send message to server
              // send({ data: text, id: "xyz.unavi.world.chat.send" });
            }
          }}
          onFocus={() => usePlayStore.setState({ chatBoxFocused: true })}
          onBlur={() => usePlayStore.setState({ chatBoxFocused: false })}
          type="text"
          maxLength={100}
          placeholder={
            chatBoxFocused ? "Send a message..." : "Press ENTER to chat"
          }
          className="h-full w-full rounded-lg bg-black/50 px-4 text-white outline-none transition placeholder:text-white/80 hover:bg-black/60 focus:bg-black/70 focus:backdrop-blur-lg focus:placeholder:text-white/75"
        />
      </div>
    </div>
  );
}
