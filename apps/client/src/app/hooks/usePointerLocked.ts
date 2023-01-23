import { useEffect, useState } from "react";

export function usePointerLocked() {
  const [isPointerLocked, setIsPointerLocked] = useState(false);

  useEffect(() => {
    const onPointerLockChange = () => {
      setIsPointerLocked(document.pointerLockElement !== null);
    };

    document.addEventListener("pointerlockchange", onPointerLockChange);
    return () => document.removeEventListener("pointerlockchange", onPointerLockChange);
  }, []);

  return isPointerLocked;
}
