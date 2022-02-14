import { useContext, useEffect } from "react";
import { Canvas } from "@react-three/fiber";
import { useContextBridge } from "@react-three/drei";
import { Physics } from "@react-three/cannon";
import { useRouter } from "next/router";
import Head from "next/head";
import { CeramicContext, useRoom, useWorld } from "ceramic";
import { MultiplayerProvider, Player, RAYCASTER_SETTINGS, Room } from "3d";

export default function App() {
  const ContextBridge = useContextBridge(CeramicContext);

  const { authenticated, connect } = useContext(CeramicContext);

  useEffect(() => {
    if (!authenticated) connect();
  }, [authenticated, connect]);

  const router = useRouter();
  const roomId = router.query.room as string;

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
              <Player spawn={world.spawn} />
              <Room roomId={roomId} />
            </Physics>
          </MultiplayerProvider>
        </ContextBridge>
      </Canvas>
    </div>
  );
}
