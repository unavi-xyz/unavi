import { useEffect } from "react";
import { Canvas } from "@react-three/fiber";
import Split from "react-split";

import { useStore } from "../src/state/useStore";
import { useScene } from "../src/state/useScene";
import { useHotkeys } from "../src/hooks/useHotkeys";

import Scene from "../src/scene/Scene";
import Navbar from "../src/ui/navbar/Navbar";
import RightPanel from "../src/ui/panel/RightPanel";

export default function Editor() {
  const setSelected = useStore((state) => state.setSelected);
  const setRoomId = useStore((state) => state.setRoomId);

  const setScene = useScene((state) => state.setScene);

  useHotkeys();

  function clearSelected() {
    setSelected(null, null);
  }

  useEffect(() => {
    const queryString = window.location.search;
    const urlParams = new URLSearchParams(queryString);
    setRoomId(urlParams.get("scene"));

    setScene({});
    setSelected(null, null);
  }, [setRoomId, setScene, setSelected]);

  return (
    <div className="App">
      <Navbar />

      <Split
        className="split App"
        gutterSize={6}
        sizes={[80, 20]}
        gutterAlign="end"
      >
        <Canvas
          onPointerMissed={clearSelected}
          gl={{ preserveDrawingBuffer: true }}
        >
          <Scene />
        </Canvas>

        <RightPanel />
      </Split>
    </div>
  );
}
