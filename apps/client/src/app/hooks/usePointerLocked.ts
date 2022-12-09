import { useEffect, useState } from "react";

export function usePointerLocked() {
  const [isPointerLocked, setIsPointerLocked] = useState(false);

  useEffect(() => {
    function handlePointerLockChange() {
      setIsPointerLocked(document.pointerLockElement !== null);
    }

    document.addEventListener("pointerlockchange", handlePointerLockChange);
    return () => {
      document.removeEventListener("pointerlockchange", handlePointerLockChange);
    };
  }, []);

  return isPointerLocked;
}
