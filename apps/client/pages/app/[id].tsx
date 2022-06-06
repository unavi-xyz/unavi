import { Physics } from "@react-three/cannon";
import { useContextBridge } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { useRouter } from "next/router";
import { Context } from "urql";

import { InstancedScene } from "@wired-xr/scene";

import Chat from "../../src/components/app/Chat";
import ConnectionProvider, {
  ConnectionContext,
} from "../../src/components/app/ConnectionProvider";
import MultiplayerManager from "../../src/components/app/MultiplayerManager";
import Player from "../../src/components/app/Player";
import MetaTags from "../../src/components/ui/MetaTags";
import { useAppHotkeys } from "../../src/helpers/app/hooks/useAppHotkeys";
import { useLoadAssets } from "../../src/helpers/app/hooks/useLoadAssets";
import { useSetIdentity } from "../../src/helpers/app/hooks/useSetIdentity";
import { usePublication } from "../../src/helpers/lens/hooks/usePublication";

export default function App() {
  const router = useRouter();
  const id = router.query.id as string;

  const publication = usePublication(id);
  const { loadedScene, spawn } = useLoadAssets(publication?.metadata.content);

  useAppHotkeys();
  useSetIdentity();

  return (
    <>
      <MetaTags title={publication?.metadata.name ?? id ?? "App"} />

      {loadedScene && publication && (
        <ConnectionProvider>
          <div className="h-full">
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
        </ConnectionProvider>
      )}
    </>
  );
}

function CanvasBridge({ children }: { children: React.ReactNode }) {
  const ContextBridge = useContextBridge(ConnectionContext, Context);
  return (
    <Canvas shadows>
      <ContextBridge>{children}</ContextBridge>
    </Canvas>
  );
}
