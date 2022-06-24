import { Physics } from "@react-three/cannon";
import { useContextBridge } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import { NextPageContext } from "next";
import { Context } from "urql";

import {
  NetworkingContext,
  NetworkingProvider,
  Player,
  PlayerManager,
  Scene,
} from "@wired-xr/engine";

import Chat from "../../src/app/Chat";
import { useAppHotkeys } from "../../src/app/hooks/useAppHotkeys";
import { useLoadAssets } from "../../src/app/hooks/useLoadAssets";
import { useSetIdentity } from "../../src/app/hooks/useSetIdentity";
import {
  PublicationProps,
  getPublicationProps,
} from "../../src/lib/lens/getPublicationProps";
import { useLensStore } from "../../src/lib/lens/store";
import MetaTags from "../../src/ui/MetaTags";

export const DEFAULT_HOST = "wss://host.thewired.space";

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

  const ownerHost = publication?.profile.attributes?.find(
    (item) => item.key === "host"
  )?.value;

  const host =
    process.env.NODE_ENV === "development"
      ? "ws://localhost:4000"
      : ownerHost
      ? `wss://${ownerHost}`
      : DEFAULT_HOST;

  useAppHotkeys();
  useSetIdentity();

  const handle = useLensStore((state) => state.handle);

  return (
    <>
      <MetaTags
        title={metadata.title}
        description={metadata.description}
        image={metadata.image}
        card="summary_large_image"
      />

      {loadedScene && (
        <div className="h-full">
          <div className="crosshair" />

          <NetworkingProvider spaceId={id} host={host} handle={handle}>
            <Chat />

            <CanvasBridge>
              <Physics>
                <Player spawn={spawn} />
                <Scene scene={loadedScene} />

                <PlayerManager
                  animationsUrl="/models/animations.fbx"
                  defaultAvatarUrl="/models/avatar.vrm"
                />
              </Physics>
            </CanvasBridge>
          </NetworkingProvider>
        </div>
      )}
    </>
  );
}

function CanvasBridge({ children }: { children: React.ReactNode }) {
  const ContextBridge = useContextBridge(NetworkingContext, Context);

  return (
    <Canvas shadows>
      <ContextBridge>{children}</ContextBridge>
    </Canvas>
  );
}
