import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { useContextBridge } from "@react-three/drei";
import { Vector3 } from "three";
import { useRouter } from "next/router";
import { CeramicContext, useRoom, useScene } from "ceramic";
import {
  ASSET_NAMES,
  Multiplayer,
  MultiplayerContext,
  Player,
  RAYCASTER_SETTINGS,
  Scene,
  SceneObject,
} from "3d";

function getSpawn(scene: SceneObject[]) {
  if (!scene) return;

  const object = scene.find((obj) => obj.type === ASSET_NAMES.Spawn);

  if (!object) return new Vector3(0, 2, 0);

  const spawn = new Vector3().fromArray(object.position);
  spawn.add(new Vector3(0, 2, 0));
  return spawn;
}

export default function App() {
  const ContextBridge = useContextBridge(CeramicContext, MultiplayerContext);

  const router = useRouter();
  const roomId = router.query.room as string;

  const room = useRoom(roomId);
  const { scene } = useScene(room?.sceneStreamId);

  if (!scene) return <div>Loading...</div>;

  return (
    <div className="App">
      <Canvas raycaster={RAYCASTER_SETTINGS}>
        <ContextBridge>
          <Physics>
            <Player spawn={scene.spawn} />
            <Scene objects={scene?.objects} />
            <Multiplayer roomId={roomId} />
          </Physics>
        </ContextBridge>
      </Canvas>
    </div>
  );
}
