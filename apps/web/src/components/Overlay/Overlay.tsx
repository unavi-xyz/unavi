import { useEffect, useState } from "react";
import Sidebar from "./Sidebar/Sidebar";

export default function Overlay() {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    function onChange() {
      setVisible(Boolean(document.pointerLockElement));
    }

    function onKeydown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        setVisible((prev) => !prev);
      }
    }

    document.addEventListener("keydown", onKeydown);
    document.addEventListener("pointerlockchange", onChange);

    return () => {
      document.removeEventListener("pointerlockchange", onChange);
      document.removeEventListener("keydown", onKeydown);
    };
  }, []);

  return (
    <div>
      <Sidebar visible={visible} />
    </div>
  );
}
