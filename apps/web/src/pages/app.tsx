import { useEffect } from "react";
import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { useRouter } from "next/router";
import { Player, World } from "3d";
import { useAuth, useRoom } from "ceramic";

import AppLayout from "../layouts/AppLayout";
import Multiplayer from "../components/app/Multiplayer";
import MultiplayerProvider from "../components/app/MultiplayerContext";
import { useSceneLoader } from "../components/app/useSceneLoader";

export default function App() {
  const router = useRouter();
  const roomId = router.query.room as string;

  const { room } = useRoom(roomId);
  const { authenticated, connect } = useAuth();

  const textures = useSceneLoader(room?.scene);

  useEffect(() => {
    if (!authenticated) connect();
  }, [authenticated, connect]);

  if (!authenticated) return null;

  return (
    <div className="h-full">
      <div className="crosshair" />

      <Canvas mode="concurrent">
        <MultiplayerProvider>
          <Physics>
            <World scene={room.scene} textures={textures} />
            <Multiplayer roomId={roomId} />
            <Player />
          </Physics>
        </MultiplayerProvider>
      </Canvas>
    </div>
  );
}

App.Layout = AppLayout;
