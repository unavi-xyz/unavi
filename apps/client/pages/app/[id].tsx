import { Physics } from "@react-three/cannon";
import { useContextBridge } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import Head from "next/head";
import { useRouter } from "next/router";

import { InstancedScene } from "@wired-xr/scene";

import Chat from "../../src/components/app/Chat";
import MultiplayerManager from "../../src/components/app/MultiplayerManager";
import Player from "../../src/components/app/Player";
import SpaceProvider, {
  SpaceContext,
} from "../../src/components/app/SpaceProvider";
import { useAppHotkeys } from "../../src/helpers/app/hooks/useAppHotkeys";
import { useLoadAssets } from "../../src/helpers/app/hooks/useLoadAssets";
import { usePublication } from "../../src/helpers/lens/hooks/usePublication";

export default function App() {
  const router = useRouter();
  const id = router.query.id as string;

  const publication = usePublication(id);
  const { loadedScene, spawn } = useLoadAssets(publication?.metadata.content);

  useAppHotkeys();

  if (!loadedScene || !publication) return null;

  return (
    <SpaceProvider spaceId={id}>
      <div className="h-full">
        <Head>
          <title>
            {publication.metadata.name ?? publication.id} / The Wired
          </title>
        </Head>

        <div className="crosshair" />

        <Chat />

        <CanvasBridge>
          <Physics>
            <InstancedScene scene={loadedScene} />

            <Player spawn={spawn} />
            <MultiplayerManager spaceId={id} />
          </Physics>
        </CanvasBridge>
      </div>
    </SpaceProvider>
  );
}

function CanvasBridge({ children }: { children: React.ReactNode }) {
  const ContextBridge = useContextBridge(SpaceContext);
  return (
    <Canvas shadows>
      <ContextBridge>{children}</ContextBridge>
    </Canvas>
  );
}
