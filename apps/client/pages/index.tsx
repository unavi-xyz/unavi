import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { Stats } from "@react-three/drei";

import { ClientProvider, MultiplayerProvider } from "matrix";
import { RAYCASTER_SETTINGS } from "3d";
import Scene from "../src/scene/Scene";

export default function App() {
  return (
    <div className="App">
      <Canvas raycaster={RAYCASTER_SETTINGS}>
        <ClientProvider>
          <MultiplayerProvider>
            <Physics>
              <Scene />
            </Physics>
          </MultiplayerProvider>
        </ClientProvider>

        <Stats />
      </Canvas>
    </div>
  );
}
