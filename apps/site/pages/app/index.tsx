import { useContext, useEffect } from "react";
import { Canvas } from "@react-three/fiber";
import { useRouter } from "next/router";
import Head from "next/head";
import { CeramicContext, useRoom, useWorld } from "ceramic";
import { Player, RAYCASTER_SETTINGS, Room } from "3d";

export default function App() {
  const { userId, authenticated, connect } = useContext(CeramicContext);

  useEffect(() => {
    if (!authenticated) connect();
  }, [authenticated, connect]);

  const router = useRouter();
  const roomId = router.query.room as string;

  const { room } = useRoom(roomId);
  const { world } = useWorld(room?.worldStreamId);

  if (!world || !authenticated) return <div>Loading...</div>;

  return (
    <div className="App">
      <Head>
        <title>The Wired - App</title>
      </Head>

      <div className="crosshair" />

      <Canvas raycaster={RAYCASTER_SETTINGS}>
        <Room roomId={roomId} userId={userId}>
          <Player spawn={world.spawn} />
        </Room>
      </Canvas>
    </div>
  );
}
