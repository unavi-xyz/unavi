import { useEffect } from "react";
import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { useRouter } from "next/router";
import { Player, World } from "3d";
import { useAuth, useSpace } from "ceramic";

import AppLayout from "../layouts/AppLayout";
import Multiplayer from "../components/app/Multiplayer";
import MultiplayerProvider from "../components/app/MultiplayerContext";
import { useSceneLoader } from "../components/app/useSceneLoader";

export default function App() {
  const router = useRouter();
  const spaceId = router.query.space as string;

  const { space } = useSpace(spaceId);
  const { authenticated, connect } = useAuth();

  const { textures, models } = useSceneLoader(space?.scene);

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
            <World scene={space.scene} textures={textures} models={models} />
            <Multiplayer spaceId={spaceId} />
            <Player />
          </Physics>
        </MultiplayerProvider>
      </Canvas>
    </div>
  );
}

App.Layout = AppLayout;
