import { useEffect } from "react";

import { useStore } from "../helpers/store";

import Profile from "./Profile";
import Chat from "./Chat";
import Mic from "./Mic";

export default function AppOverlay() {
  const isPointerLocked = useStore((state) => state.isPointerLocked);

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

      <div className="absolute w-full h-full top-0 left-0 flex justify-end">
        {!isPointerLocked && <Profile />}
      </div>

      <div className="absolute w-full h-full top-0 left-0 flex items-end">
        <div className="w-full">
          <Chat />
        </div>
        <div className="w-full">{!isPointerLocked && <Mic />}</div>
        <div className="w-full"></div>
      </div>
    </div>
  );
}
