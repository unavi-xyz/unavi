import { createProxySSGHelpers } from "@trpc/react-query/ssg";
import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import Script from "next/script";
import { useEffect, useRef } from "react";

import { useAnalytics } from "../../app/hooks/useAnalytics";
import { useAppHotkeys } from "../../app/hooks/useAppHotkeys";
import { useLoadUser } from "../../app/hooks/useLoadUser";
import { useResizeEngineCanvas } from "../../app/hooks/useResizeEngineCanvas";
import { useSetAvatar } from "../../app/hooks/useSetAvatar";
import { useSpace } from "../../app/hooks/useSpace";
import { useAppStore } from "../../app/store";
import ChatBox from "../../app/ui/ChatBox";
import LoadingScreen from "../../app/ui/LoadingScreen";
import MobileChatBox from "../../app/ui/MobileChatBox";
import UserButton from "../../app/ui/UserButtons";
import MetaTags from "../../home/MetaTags";
import { prisma } from "../../server/prisma";
import { appRouter } from "../../server/router/_app";
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

  const ssg = await createProxySSGHelpers({
    router: appRouter,
    ctx: {
      prisma,
      res,
      session: null,
    },
  });

  await ssg.space.byId.prefetch({ id });

  return {
    props: {
      trpcState: ssg.dehydrate(),
      id,
    },
  };
};

export default function App({ id }: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const createdEngine = useRef(false);

  const engine = useAppStore((state) => state.engine);

  useResizeEngineCanvas(engine, canvasRef, containerRef);
  useLoadUser();
  useAppHotkeys();
  useAnalytics();
  const setAvatar = useSetAvatar();
  const isMobile = useIsMobile();
  const { space, loadingText, loadingProgress, join } = useSpace(id);

  useEffect(() => {
    if (createdEngine.current) return;
    createdEngine.current = true;

    async function initEngine() {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error("Canvas not found");

      const { Engine } = await import("engine");

      // Create engine
      const engine = new Engine({
        canvas,
        camera: "player",
        skyboxPath: "/images/skybox/",
        avatarPath: "/models/Wired-chan.vrm",
        avatarAnimationsPath: "/models/",
      });

      await engine.start();

      useAppStore.setState({ engine });
    }

    initEngine();
  }, [canvasRef]);

  useEffect(() => {
    if (!engine) return;

    join();

    return () => {
      engine.destroy();
      useAppStore.setState({ engine: null, chatMessages: [] });
    };
  }, [engine, join]);

  const loaded = loadingProgress === 1;
  const loadedClass = loaded ? "opacity-100" : "opacity-0";

  return (
    <>
      <MetaTags
        title={space?.metadata?.name}
        description={space?.metadata?.description}
        image={space?.metadata?.image}
        card="summary_large_image"
      />

      <Script src="/scripts/draco_decoder.js" />

      <LoadingScreen
        text={space?.metadata?.name}
        image={space?.metadata?.image}
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
        {loaded && (
          <div className="absolute inset-x-0 top-0 z-10 mx-auto mt-4 w-96">
            <UserButton />
          </div>
        )}

        <div className="h-full">
          <div ref={containerRef} className="relative h-full w-full overflow-hidden">
            <canvas ref={canvasRef} className={`h-full w-full transition ${loadedClass}`} />
          </div>
        </div>

        {loaded ? (
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
