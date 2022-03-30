import { useEffect, useState } from "react";
import { Message } from "../helpers/types";

const showDuration = 10000;

function isFading(message: Message) {
  return Date.now() - message.time > showDuration;
}

function isHidden(message: Message) {
  return Date.now() - message.time > showDuration + 2000;
}

interface Props {
  message: Message;
}

export default function ChatMessage({ message }: Props) {
  const { text, username } = message;

  const [hidden, setHidden] = useState(isHidden(message));
  const [fading, setFading] = useState(isFading(message));
  const [entering, setEntering] = useState(true);

  useEffect(() => {
    setTimeout(() => {
      setEntering(false);
    }, 5);
  }, []);

  useEffect(() => {
    if (hidden) return;

    const interval = setInterval(() => {
      const newFading = isFading(message);
      const newHidden = isHidden(message);

      if (newFading !== fading) setFading(newFading);
      if (newHidden !== hidden) setHidden(newHidden);
    }, 100);

    return () => {
      clearInterval(interval);
    };
  }, [message, hidden, fading]);

  const exitCss = entering || fading ? "-translate-x-full opacity-0" : "";

  if (hidden) return null;

  return (
    <div
      className={`flex items-center space-x-2 py-2 px-2 bg-white w-min min-w-fit
                  rounded-lg text-sm transition-all duration-700 ${exitCss}`}
    >
      {username && <div className="break-normal">{username}:</div>}
      <div className="break-words">{text}</div>
    </div>
  );
}
