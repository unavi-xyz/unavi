import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { useContextBridge } from "@react-three/drei";
import { useRouter } from "next/router";
import Head from "next/head";
import { CeramicContext, useRoom, useScene } from "ceramic";
import {
  Multiplayer,
  MultiplayerProvider,
  Player,
  RAYCASTER_SETTINGS,
  Scene,
} from "3d";

export default function App() {
  const ContextBridge = useContextBridge(CeramicContext);

  const router = useRouter();
  const roomId = router.query.room as string;

  const room = useRoom(roomId);
  const { scene } = useScene(room?.sceneStreamId);

  if (!scene) return <div>Loading...</div>;

  return (
    <div className="App">
      <Head>
        <title>The Wired - App</title>
      </Head>

      <Canvas raycaster={RAYCASTER_SETTINGS}>
        <ContextBridge>
          <MultiplayerProvider>
            <Physics>
              <Player spawn={scene.spawn} />
              <Scene objects={scene?.objects} />
              <Multiplayer roomId={roomId} />
            </Physics>
          </MultiplayerProvider>
        </ContextBridge>
      </Canvas>
    </div>
  );
}
