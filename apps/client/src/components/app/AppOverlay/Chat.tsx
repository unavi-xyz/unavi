import { KeyboardEvent, useContext, useEffect, useRef } from "react";
import { nanoid } from "nanoid";

import { appManager, useStore } from "../helpers/store";
import { Message } from "../helpers/types";
import { SocketContext } from "../SocketProvider";

import ChatMessage from "./ChatMessage";

export default function Chat() {
  const inputRef = useRef<HTMLInputElement>();

  const messages = useStore((state) => state.messages);

  const { socket } = useContext(SocketContext);

  useEffect(() => {
    appManager.setChatInputRef(inputRef);
  }, []);

  function handleKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") {
      const target = e.target as HTMLInputElement;
      const text = target.value;
      const time = Date.now();
      const message: Message = {
        id: nanoid(),
        username: socket.id,
        text,
        time,
      };

      if (text === "") return;

      appManager.publishMessage(message);
      target.value = "";
    }
  }

  return (
    <div className="p-8 w-full space-y-4">
      <div className="w-1/3 overflow-y-hidden flex flex-col-reverse">
        {messages.map((message) => {
          return (
            <div key={message.id}>
              <ChatMessage message={message} />
            </div>
          );
        })}
      </div>

      <input
        ref={inputRef}
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
