import { Physics } from "@react-three/cannon";
import { useContextBridge } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { NextPageContext } from "next";
import { Context } from "urql";

import { InstancedScene } from "@wired-xr/scene";

import Chat from "../../src/components/app/Chat";
import Player from "../../src/components/app/Player";
import PlayerManager from "../../src/components/app/PlayerManager";
import SpaceProvider, {
  SpaceContext,
} from "../../src/components/app/SpaceProvider";
import MetaTags from "../../src/components/ui/MetaTags";
import { useAppHotkeys } from "../../src/helpers/app/hooks/useAppHotkeys";
import { useLoadAssets } from "../../src/helpers/app/hooks/useLoadAssets";
import { useSetIdentity } from "../../src/helpers/app/hooks/useSetIdentity";
import {
  PublicationProps,
  getPublicationProps,
} from "../../src/helpers/lens/getPublicationProps";

export async function getServerSideProps({ res, query }: NextPageContext) {
  res?.setHeader("Cache-Control", "s-maxage=120");

  const id = query.id as string;
  const props = await getPublicationProps(id);

  return {
    props: {
      ...props,
      id,
    },
  };
}

interface Props extends PublicationProps {
  id: string;
}

export default function App({ id, metadata, publication }: Props) {
  const { loadedScene, spawn } = useLoadAssets(publication?.metadata.content);

  useAppHotkeys();
  useSetIdentity();

  return (
    <>
      <MetaTags
        title={metadata.title}
        description={metadata.description}
        image={metadata.image}
        imageWidth="595.2px"
        imageHeight="357.11px"
        card="summary_large_image"
      />

      {loadedScene && publication && (
        <SpaceProvider spaceId={id}>
          <div className="h-full">
            <div className="crosshair" />

            <Chat />

            <CanvasBridge>
              <Physics>
                <InstancedScene scene={loadedScene} />
                <PlayerManager />

                <Player spawn={spawn} />
              </Physics>
            </CanvasBridge>
          </div>
        </SpaceProvider>
      )}
    </>
  );
}

function CanvasBridge({ children }: { children: React.ReactNode }) {
  const ContextBridge = useContextBridge(SpaceContext, Context);
  return (
    <Canvas shadows>
      <ContextBridge>{children}</ContextBridge>
    </Canvas>
  );
}
