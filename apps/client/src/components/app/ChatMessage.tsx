import { useEffect, useState } from "react";

import { IChatMessage } from "../../helpers/app/types";

const showDuration = 10000;

function isFading(message: IChatMessage) {
  return Date.now() - message.timestamp / 1000000.0 > showDuration;
}

function isHidden(message: IChatMessage) {
  return Date.now() - message.timestamp / 1000000.0 > showDuration + 2000;
}

interface Props {
  message: IChatMessage;
}

export default function ChatMessage({ message }: Props) {
  const { message: text, username } = message;

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
      onClick={(e) => e.stopPropagation()}
      className={`flex items-center space-x-2 my-1 py-2 px-2 bg-surface text-onSurface
                  w-min min-w-fit rounded-lg text-sm transition-all duration-700 ${exitCss}`}
    >
      {username && <div className="break-normal">{username}:</div>}
      <div className="break-words">{text}</div>
    </div>
  );
}
