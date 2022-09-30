import { NextPageContext } from "next";

import {
  getPublicationProps,
  PublicationProps,
} from "../../lib/lens/utils/getPublicationProps";
import MetaTags from "../../ui/MetaTags";

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

export default function App({ id, metadata }: Props) {
  // const ownerHost = publication?.profile.attributes?.find(
  //   (item) => item.key === "host"
  // )?.value;

  // const host =
  //   process.env.NODE_ENV === "development"
  //     ? "ws://localhost:4000"
  //     : ownerHost
  //     ? `wss://${ownerHost}`
  //     : DEFAULT_HOST;

  // useAppHotkeys();
  // useSetIdentity();

  // useEffect(() => {
  //   //send an analytics event when the user joins the room
  //   //so we can show popular spaces on the explore page
  //   //idk if this is a good way to do it but it works for now
  //   if (process.env.NODE_ENV === "production") {
  //     // fetch(`/api/space/${id}/add-view`);
  //   }
  // }, [id]);

  return (
    <>
      <MetaTags
        title={metadata.title ?? id}
        description={metadata.description ?? undefined}
        image={metadata.image ?? undefined}
        card="summary_large_image"
      />

      <div className="h-full">
        <div className="crosshair" />

        {/* <NetworkingProvider spaceId={id} host={host}>
            <Chat />

            <EngineCanvas>
              <Player spawn={spawn} />
              <Scene scene={loadedScene} />

              <PlayerManager
                animationsUrl="/models/animations.fbx"
                defaultAvatarUrl="/models/avatar.vrm"
              />
            </EngineCanvas>
          </NetworkingProvider> */}
      </div>
    </>
  );
}
