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
import { usePublication } from "../../src/helpers/lens/hooks/usePublication";

export default function App() {
  const router = useRouter();
  const id = router.query.id as string;

  const publication = usePublication(id);
  const scene = publication?.metadata.content;

  useAppHotkeys();

  if (!scene) return null;

  return (
    <SpaceProvider spaceId={id}>
      <div className="h-full">
        <Head>
          <title>
            {publication.metadata.name ?? publication.id} · The Wired
          </title>
        </Head>

        <div className="crosshair" />

        <Chat />

        <Bridge>
          <InstancedScene scene={JSON.parse(scene)}>
            <Player />
            <MultiplayerManager spaceId={id} />
          </InstancedScene>
        </Bridge>
      </div>
    </SpaceProvider>
  );
}

function Bridge({ children }: { children: React.ReactNode }) {
  const ContextBridge = useContextBridge(SpaceContext);
  return (
    <Canvas>
      <ContextBridge>{children}</ContextBridge>
    </Canvas>
  );
}
