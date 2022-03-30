import { MouseEvent, useContext } from "react";
import { BsMicFill, BsMicMuteFill } from "react-icons/bs";

import { appManager, useStore } from "../helpers/store";
import { SocketContext } from "../SocketProvider";

export default function Mic() {
  const muted = useStore((state) => state.muted);

  const { getUserMedia } = useContext(SocketContext);

  function handleClick(e: MouseEvent) {
    getUserMedia();

    e.stopPropagation();
    appManager.setMuted(!muted);
  }

  return (
    <div className="flex justify-center p-8">
      <div
        onClick={handleClick}
        className="rounded-xl p-4 bg-white cursor-pointer hover:bg-neutral-200 transition-all"
      >
        {muted ? (
          <BsMicMuteFill className="text-lg text-red-600" />
        ) : (
          <BsMicFill className="text-lg" />
        )}
      </div>
    </div>
  );
}
