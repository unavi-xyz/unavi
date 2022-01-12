import { useEffect } from "react";
import { Canvas } from "@react-three/fiber";
import Split from "react-split";

import { useStore } from "../src/hooks/useStore";
import { useScene } from "../src/hooks/useScene";
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
  }, [setRoomId, setScene]);

  return (
    <div className="App">
      <Navbar />

      <Split
        className="split App"
        gutterSize={6}
        sizes={[80, 20]}
        gutterAlign="end"
      >
        <Canvas onPointerMissed={clearSelected}>
          <Scene />
        </Canvas>

        <RightPanel />
      </Split>
    </div>
  );
}
