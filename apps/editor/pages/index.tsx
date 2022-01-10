import { Canvas } from "@react-three/fiber";
import Split from "react-split";

import { RAYCASTER_SETTINGS } from "3d";
import Scene from "../src/scene/Scene";
import Navbar from "../src/ui/navbar/Navbar";
import RightPanel from "../src/ui/RightPanel";

export default function App() {
  return (
    <div className="App">
      <Navbar />

      <Split className="split App" gutterSize={6} sizes={[70, 30]}>
        <Canvas raycaster={RAYCASTER_SETTINGS}>
          <Scene />
        </Canvas>

        <RightPanel />
      </Split>
    </div>
  );
}
