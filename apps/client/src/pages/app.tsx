import { useEffect } from "react";
import { Canvas } from "@react-three/fiber";
import { Physics } from "@react-three/cannon";
import { useRouter } from "next/router";
import { Player, InstancedScene } from "3d";
import { useAuth, useSpace } from "ceramic";

import { useAssetLoader } from "../components/app/useAssetLoader";

import AppLayout from "../layouts/AppLayout";
import Multiplayer from "../components/app/Multiplayer";
import MultiplayerProvider from "../components/app/MultiplayerContext";

export default function App() {
  const router = useRouter();
  const spaceId = router.query.space as string;

  const { space } = useSpace(spaceId);
  const { authenticated, connect } = useAuth();

  const scene = useAssetLoader(space?.scene);

  useEffect(() => {
    if (!authenticated) connect();
  }, [authenticated, connect]);

  if (!authenticated || !scene) return null;

  return (
    <div className="h-full">
      <div className="crosshair" />

      <Canvas mode="concurrent">
        <MultiplayerProvider>
          <Physics>
            <InstancedScene scene={scene} />
            <Multiplayer spaceId={spaceId} />
            <Player />
          </Physics>
        </MultiplayerProvider>
      </Canvas>
    </div>
  );
}

App.Layout = AppLayout;
