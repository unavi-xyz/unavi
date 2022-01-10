import { Canvas } from "@react-three/fiber";

import { RAYCASTER_SETTINGS } from "3d";
import Scene from "../src/scene/Scene";

export default function App() {
  return (
    <div className="App">
      <Canvas raycaster={RAYCASTER_SETTINGS}>
        <Scene />
      </Canvas>
    </div>
  );
}
