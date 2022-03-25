import { KeyboardEvent, useContext, useEffect, useRef } from "react";

import { appManager, useStore } from "../helpers/store";
import { MultiplayerContext } from "../MultiplayerProvider";
import ChatMessage from "./ChatMessage";

export default function Chat() {
  const inputRef = useRef<HTMLInputElement>();

  const messages = useStore((state) => state.messages);

  const { publishMessage: sendChatMessage, getMessages } =
    useContext(MultiplayerContext);

  useEffect(() => {
    appManager.setChatInputRef(inputRef);
  }, []);

  useEffect(() => {
    const interval = setInterval(getMessages, 500);
    return () => {
      clearInterval(interval);
    };
  }, [getMessages]);

  function handleKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") {
      const target = e.target as HTMLInputElement;
      const text = target.value;
      sendChatMessage(text);
      target.value = "";
    }
  }

  return (
    <div
      onClick={(e) => e.stopPropagation()}
      className="p-8 max-w-xl space-y-4"
    >
      <div className="rounded-xl space-y-2">
        {messages.map((message) => {
          return <ChatMessage key={message.id} message={message} />;
        })}
      </div>

      <input
        ref={inputRef}
        onKeyDown={handleKeyDown}
        type="text"
        maxLength={420}
        placeholder="Press T to chat"
        className="px-3 py-2 rounded outline-none w-full bg-neutral-500
                     bg-opacity-20 backdrop-blur-xl focus:bg-white text-sm"
      />
    </div>
  );
}
