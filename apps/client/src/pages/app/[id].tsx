import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import Script from "next/script";
import { useEffect, useRef, useState } from "react";

import { useAnalytics } from "../../app/hooks/useAnalytics";
import { useAppHotkeys } from "../../app/hooks/useAppHotkeys";
import { useLoadUser } from "../../app/hooks/useLoadUser";
import { useResizeEngineCanvas } from "../../app/hooks/useResizeEngineCanvas";
import { useSetAvatar } from "../../app/hooks/useSetAvatar";
import { useAppStore } from "../../app/store";
import ChatBox from "../../app/ui/ChatBox";
import LoadingScreen from "../../app/ui/LoadingScreen";
import MobileChatBox from "../../app/ui/MobileChatBox";
import UserButton from "../../app/ui/UserButtons";
import { trpc } from "../../client/trpc";
import MetaTags from "../../home/MetaTags";
import { hexDisplayToNumber } from "../../utils/numberToHexDisplay";
import { useIsMobile } from "../../utils/useIsMobile";

export const getServerSideProps = async ({ res, query }: GetServerSidePropsContext) => {
  const ONE_MINUTE_IN_SECONDS = 60;
  const ONE_WEEK_IN_SECONDS = 60 * 60 * 24 * 7;

  res.setHeader(
    "Cache-Control",
    `public, max-age=0, s-maxage=${ONE_MINUTE_IN_SECONDS}, stale-while-revalidate=${ONE_WEEK_IN_SECONDS}`
  );

  const hexId = query.id as string;
  const id = hexDisplayToNumber(hexId);

  return {
    props: { id },
  };
};

export default function App({ id }: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const createdEngine = useRef(false);
  const [engineStarted, setEngineStarted] = useState(false);
  const [loadingProgress, setLoadingProgress] = useState(0);
  const [loadingText, setLoadingText] = useState("Starting engine...");

  const engine = useAppStore((state) => state.engine);

  useResizeEngineCanvas(engine, canvasRef, containerRef);
  useLoadUser();
  useAppHotkeys();
  useAnalytics();

  const setAvatar = useSetAvatar();
  const isMobile = useIsMobile();

  const { data: space } = trpc.space.byId.useQuery(
    { id },
    {
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
    }
  );

  useEffect(() => {
    if (!engine) return;

    async function joinSpace() {
      if (!engine) return;

      setLoadingText("Fetching space...");
      setLoadingProgress(0.2);

      if (!space) return;

      // Display loading status
      engine.networking.spaceJoinStatus$.subscribe(
        ({ sceneLoaded, webrtcConnected, wsConnected }) => {
          setLoadingText("Connecting...");
          setLoadingProgress(0.35);

          if (!wsConnected) return;

          setLoadingText("Connecting...");
          setLoadingProgress(0.5);

          if (!webrtcConnected) return;

          setLoadingText("Loading scene...");
          setLoadingProgress(0.75);

          if (!sceneLoaded) return;

          setLoadingText("Ready!");
          setLoadingProgress(1);
        }
      );

      const host =
        process.env.NODE_ENV === "development"
          ? "ws://localhost:4000"
          : `wss://${process.env.NEXT_PUBLIC_DEFAULT_HOST}`;

      // Join space
      await engine.networking.joinSpace({
        spaceId: space.id,
        host,
        modelURL: space.metadata.animation_url,
      });

      // Start engine
      await engine.start();

      setEngineStarted(true);
    }

    joinSpace();

    return () => {
      engine.networking.leaveSpace();
    };
  }, [engine, space]);

  useEffect(() => {
    if (createdEngine.current) return;
    createdEngine.current = true;

    async function initEngine() {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error("Canvas not found");

      setLoadingText("Starting engine...");
      setLoadingProgress(0);

      const { Engine } = await import("engine");

      // Create engine
      const engine = new Engine({
        canvas,
        camera: "player",
        skyboxPath: "/images/skybox/",
        avatarPath: "/models/Wired-chan.vrm",
        avatarAnimationsPath: "/models/",
      });

      await engine.waitForReady();

      useAppStore.setState({ engine });
    }

    initEngine();
  }, [canvasRef]);

  useEffect(() => {
    if (!engine) return;

    return () => {
      engine.destroy();
      useAppStore.setState({ engine: null });
    };
  }, [engine]);

  const loadedClass = engineStarted ? "opacity-100" : "opacity-0";

  return (
    <>
      <MetaTags
        title={space?.metadata.name}
        description={space?.metadata.description}
        image={space?.metadata.image}
        card="summary_large_image"
      />

      <Script src="/scripts/draco_decoder.js" />

      <LoadingScreen
        text={space?.metadata.name}
        image={space?.metadata.image}
        loaded={engineStarted}
        loadingProgress={loadingProgress}
        loadingText={loadingText}
      />

      <div
        className="h-screen w-screen"
        onDragOver={(e) => e.preventDefault()}
        onDrop={(e) => {
          const { engine } = useAppStore.getState();
          if (!engine) return;

          e.preventDefault();
          const item = e.dataTransfer.items[0];

          if (item?.kind !== "file") return;

          const file = item.getAsFile();
          if (!file) return;

          const isVRM = file.name.endsWith(".vrm");
          if (!isVRM) return;

          // Set avatar
          const url = URL.createObjectURL(file);
          setAvatar(url);

          useAppStore.setState({ customAvatar: url });
        }}
      >
        {engineStarted && (
          <div className="absolute inset-x-0 top-0 z-10 mx-auto mt-4 w-96">
            <UserButton />
          </div>
        )}

        <div className="h-full">
          <div ref={containerRef} className="relative h-full w-full overflow-hidden">
            <canvas ref={canvasRef} className={`h-full w-full transition ${loadedClass}`} />
          </div>
        </div>

        {engineStarted ? (
          isMobile ? (
            <div className="absolute left-0 bottom-0 z-10 p-4">
              <MobileChatBox />
            </div>
          ) : (
            <div className="absolute left-0 bottom-0 z-10 w-full max-w-sm p-4">
              <ChatBox />
            </div>
          )
        ) : null}
      </div>
    </>
  );
}
