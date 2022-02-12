import { useContext, useEffect, useRef } from "react";
import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { useContextBridge } from "@react-three/drei";
import { useRouter } from "next/router";
import Head from "next/head";
import { Group } from "three";
import { CeramicContext, useRoom, useWorld } from "ceramic";
import {
  Multiplayer,
  MultiplayerProvider,
  Player,
  RAYCASTER_SETTINGS,
  Scene,
} from "3d";

export default function App() {
  const ContextBridge = useContextBridge(CeramicContext);

  const { authenticated, connect } = useContext(CeramicContext);

  useEffect(() => {
    if (!authenticated) connect();
  }, [authenticated, connect]);

  const router = useRouter();
  const roomId = router.query.room as string;

  const floor = useRef<Group>();

  const { room } = useRoom(roomId);
  const { world } = useWorld(room?.worldStreamId);

  if (!world) return <div>Loading...</div>;

  return (
    <div className="App">
      <Head>
        <title>The Wired - App</title>
      </Head>

      <div className="crosshair" />

      <Canvas raycaster={RAYCASTER_SETTINGS}>
        <ContextBridge>
          <MultiplayerProvider>
            <Physics>
              <Player world={floor} spawn={world.spawn} />
              <Multiplayer roomId={roomId} />
              <group ref={floor}>
                <Scene objects={world?.scene} />
              </group>
            </Physics>
          </MultiplayerProvider>
        </ContextBridge>
      </Canvas>
    </div>
  );
}
