import { MouseEvent, useContext } from "react";
import { BsMicFill, BsMicMuteFill } from "react-icons/bs";

import { useStore } from "../helpers/store";
import { SocketContext } from "../SocketProvider";

export default function Mic() {
  const muted = useStore((state) => state.muted);

  const { getUserMedia } = useContext(SocketContext);

  function handleClick(e: MouseEvent) {
    getUserMedia();

    e.stopPropagation();
    useStore.setState({ muted: !muted });
  }

  return (
    <div className="flex w-full justify-center p-8">
      <div
        onClick={handleClick}
        className="rounded-xl w-12 h-12 bg-white cursor-pointer hover:bg-neutral-200 transition-all
                   z-10 flex items-center justify-center"
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
