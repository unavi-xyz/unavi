import { useEffect } from "react";

import { useStore } from "../helpers/store";

import Chat from "./Chat";
import Mic from "./Mic";
import SidePanel from "./SidePanel/SidePanel";

export default function AppOverlay() {
  useEffect(() => {
    function onPointerLockChange() {
      if (document.pointerLockElement) {
        useStore.setState({ isPointerLocked: true });
      } else {
        useStore.setState({ isPointerLocked: false });
      }
    }

    document.addEventListener("pointerlockchange", onPointerLockChange);
    return () => {
      document.removeEventListener("pointerlockchange", onPointerLockChange);
    };
  }, []);

  return (
    <div>
      <div className="crosshair" />

      <div
        className={`absolute w-full h-full top-0 left-0 flex justify-end overflow-hidden`}
      >
        <SidePanel />
      </div>

      <div className="absolute w-full h-full top-0 left-0 flex items-end">
        <Chat />
      </div>

      <div className="absolute w-full h-full top-0 left-0 flex items-end">
        <Mic />
      </div>
    </div>
  );
}
