import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { RAYCASTER_SETTINGS, MultiplayerProvider } from "3d";
import { CeramicProvider } from "ceramic";

import Scene from "../src/scene/Scene";

export default function App() {
  return (
    <div className="App">
      <Canvas raycaster={RAYCASTER_SETTINGS}>
        <CeramicProvider>
          <MultiplayerProvider>
            <Physics>
              <Scene />
            </Physics>
          </MultiplayerProvider>
        </CeramicProvider>
      </Canvas>
    </div>
  );
}
