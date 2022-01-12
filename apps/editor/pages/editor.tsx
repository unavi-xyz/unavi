import { Canvas } from "@react-three/fiber";
import Split from "react-split";

import { useStore } from "../src/state";

import Scene from "../src/scene/Scene";
import Navbar from "../src/ui/navbar/Navbar";
import RightPanel from "../src/ui/panel/RightPanel";

export default function Editor() {
  const setSelected = useStore((state) => state.setSelected);

  function clearSelected() {
    setSelected(null, null);
  }

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
