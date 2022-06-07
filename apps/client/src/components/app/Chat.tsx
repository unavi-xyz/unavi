import { KeyboardEvent, useContext, useEffect, useRef } from "react";

import { useAppStore } from "../../helpers/app/store";
import ChatMessage from "./ChatMessage";
import { ConnectionContext } from "./ConnectionProvider";

export default function Chat() {
  const chatInputRef = useRef<HTMLInputElement>(null);
  const focusedRef = useRef(false);

  const messages = useAppStore((state) => state.messages);
  const { publishAll } = useContext(ConnectionContext);

  useEffect(() => {
    useAppStore.setState({ chatInputRef });
  }, []);

  function handleKeyDown(e: KeyboardEvent<HTMLInputElement>) {
    if (focusedRef.current !== true) return;

    switch (e.key) {
      case "w":
      case "a":
      case "s":
      case "d":
      case "t":
      case "W":
      case "A":
      case "S":
      case "D":
      case "T":
      case " ":
        e.stopPropagation();
    }

    if (e.key === "Enter") {
      const target = e.target as HTMLInputElement;
      const text = target.value;

      if (text === "") return;

      publishAll("message", text);
      target.value = "";
    }
  }

  return (
    <div className="absolute bottom-0 p-8 w-full space-y-4 z-10">
      <div className="w-full max-w-lg overflow-y-hidden flex flex-col-reverse">
        {messages.map((message) => {
          return <ChatMessage key={message.id} message={message} />;
        })}
      </div>

      <input
        ref={chatInputRef}
        onKeyDown={handleKeyDown}
        onFocus={() => (focusedRef.current = true)}
        onBlur={() => (focusedRef.current = false)}
        type="text"
        maxLength={420}
        placeholder="Press T to chat"
        onClick={(e) => e.stopPropagation()}
        className="px-3 py-2 rounded outline-none w-full max-w-lg bg-surfaceDark
                   bg-opacity-50 backdrop-blur-xl focus:bg-opacity-100 text-sm transition
                   text-onSurfaceDark placeholder-onSurfaceDark placeholder-opacity-60"
      />
    </div>
  );
}
