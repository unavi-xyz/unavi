// import { VRCanvas } from "@react-three/xr";
import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { Stats } from "@react-three/drei";

import { RAYCASTER_SETTINGS } from "../src/constants";
import Scene from "../src/scene/Scene";

export default function App() {
  return (
    <div className="App">
      <Canvas raycaster={RAYCASTER_SETTINGS}>
        <Physics>
          <Scene />
        </Physics>

        <Stats />
      </Canvas>
    </div>
  );
}
