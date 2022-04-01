import { KeyboardEvent, useContext, useEffect, useRef } from "react";

import { appManager, useStore } from "../helpers/store";
import { SocketContext } from "../SocketProvider";

import ChatMessage from "./ChatMessage";

export default function Chat() {
  const chatInputRef = useRef<HTMLInputElement>();

  const messages = useStore((state) => state.messages);

  const { socket } = useContext(SocketContext);

  useEffect(() => {
    useStore.setState({ chatInputRef });
  }, []);

  function handleKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") {
      const target = e.target as HTMLInputElement;
      const text = target.value;

      if (text === "") return;

      appManager.publishAll("message", text);
      target.value = "";
    }
  }

  return (
    <div className="p-8 w-full space-y-4">
      <div className="w-1/3 overflow-y-hidden flex flex-col-reverse">
        {messages.map((message) => {
          return <ChatMessage key={message.id} message={message} />;
        })}
      </div>

      <input
        ref={chatInputRef}
        onKeyDown={handleKeyDown}
        type="text"
        maxLength={420}
        placeholder="Press T to chat"
        onClick={(e) => e.stopPropagation()}
        className="px-3 py-2 rounded outline-none w-full max-w-xl bg-neutral-500
                   bg-opacity-20 backdrop-blur-xl focus:bg-white text-sm"
      />
    </div>
  );
}
