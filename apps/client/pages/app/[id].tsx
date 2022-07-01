import { NextPageContext } from "next";
import { useEffect } from "react";

import {
  EngineCanvas,
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

  useEffect(() => {
    //send an analytics event when the user joins the room
    //so we can show popular spaces on the explore page
    //idk if this is a good way to do it but it works for now
    if (process.env.NODE_ENV === "production") {
      fetch(`/api/space/${id}/add-view`);
    }
  }, [id]);

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

          <NetworkingProvider spaceId={id} host={host}>
            <Chat />

            <EngineCanvas>
              <Player spawn={spawn} />
              <Scene scene={loadedScene} />

              <PlayerManager
                animationsUrl="/models/animations.fbx"
                defaultAvatarUrl="/models/avatar.vrm"
              />
            </EngineCanvas>
          </NetworkingProvider>
        </div>
      )}
    </>
  );
}
