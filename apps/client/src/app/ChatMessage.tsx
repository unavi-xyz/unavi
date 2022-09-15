import { useEffect, useState } from "react";

const showDurationMs = 10 * 1000;

function isFading(message: any) {
  return Date.now() - message.timestamp > showDurationMs;
}

function isHidden(message: any) {
  return Date.now() - message.timestamp > showDurationMs + 2000;
}

interface Props {
  message: any;
}

export default function ChatMessage({ message }: Props) {
  const { text, senderName } = message;

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
      className={`bg-surface text-onSurface my-1 flex w-min min-w-fit items-center space-x-2
                  rounded-lg p-2 text-sm transition duration-700${exitCss}`}
    >
      <div className="break-normal">{senderName}:</div>
      <div className="break-words">{text}</div>
    </div>
  );
}
