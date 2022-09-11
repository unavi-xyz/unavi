import { useState } from "react";

interface Props {
  action: any;
}

export default function AnimationControl({ action }: Props) {
  const [running, setRunning] = useState(action.isRunning());

  const clip = action.getClip();

  function onClick() {
    if (running) {
      action.stop();
    } else {
      action.play();
    }

    setRunning(!running);
  }

  const runningCss = running ? "bg-neutral-200" : null;

  return (
    <div className="grid grid-cols-2 gap-2">
      <div className="break-all">{clip.name}</div>

      <button
        onClick={onClick}
        className={`${runningCss} w-full text-center hover:bg-neutral-200 rounded transition`}
      >
        {running ? "Stop" : "Start"}
      </button>
    </div>
  );
}
