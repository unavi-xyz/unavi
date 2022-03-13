import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { useAtom } from "jotai";
import { Player, World } from "3d";

import { sceneAtom } from "../../../helpers/editor/state";

export default function PreviewCanvas() {
  const [scene] = useAtom(sceneAtom);

  return (
    <Canvas mode="concurrent">
      <Physics>
        <World scene={scene} />
        <Player />
      </Physics>
    </Canvas>
  );
}
