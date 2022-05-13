import { Canvas } from "@react-three/fiber";
import Head from "next/head";
import { useRouter } from "next/router";
import { InstancedScene } from "scene";

import MultiplayerManager from "../../src/components/app/MultiplayerManager";
import Player from "../../src/components/app/Player";
import SpaceProvider from "../../src/components/app/SpaceProvider";
import { usePublication } from "../../src/helpers/lens/hooks/usePublication";

export default function App() {
  const router = useRouter();
  const id = router.query.id as string;

  const publication = usePublication(id);
  const scene = publication?.metadata.content;

  if (!scene) return null;

  return (
    <div className="h-full">
      <Head>
        <title>{publication.metadata.name ?? publication.id} · The Wired</title>
      </Head>

      <div className="crosshair" />

      <Canvas>
        <SpaceProvider spaceId={id}>
          <InstancedScene scene={JSON.parse(scene)}>
            <Player />
            <MultiplayerManager spaceId={id} />
          </InstancedScene>
        </SpaceProvider>
      </Canvas>
    </div>
  );
}
