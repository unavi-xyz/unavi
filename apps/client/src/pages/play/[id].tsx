import { createProxySSGHelpers } from "@trpc/react-query/ssg";
import { Engine } from "engine";
import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import Script from "next/script";
import { useEffect, useRef, useState } from "react";

import { useAppHotkeys } from "../../app/hooks/useAppHotkeys";
import { useLoadUser } from "../../app/hooks/useLoadUser";
import { useResizeCanvas } from "../../app/hooks/useResizeCanvas";
import { useSetAvatar } from "../../app/hooks/useSetAvatar";
import { useSpace } from "../../app/hooks/useSpace";
import { useAppStore } from "../../app/store";
import LoadingScreen from "../../app/ui/LoadingScreen";
import Overlay from "../../app/ui/Overlay";
import MetaTags from "../../home/MetaTags";
import { prisma } from "../../server/prisma";
import { appRouter } from "../../server/router/_app";
import { hexDisplayToNumber } from "../../utils/numberToHexDisplay";

export const getServerSideProps = async ({ res, query }: GetServerSidePropsContext) => {
  const ONE_MINUTE_IN_SECONDS = 60;
  const ONE_MONTH_IN_SECONDS = 60 * 60 * 24 * 30;

  res.setHeader(
    "Cache-Control",
    `public, max-age=0, s-maxage=${ONE_MINUTE_IN_SECONDS}, stale-while-revalidate=${ONE_MONTH_IN_SECONDS}`
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

export default function Play({ id }: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);

  const engine = useAppStore((state) => state.engine);
  const [scriptsReady, setScriptsReady] = useState(false);

  const setAvatar = useSetAvatar();
  useResizeCanvas(engine, canvasRef, overlayRef, containerRef);
  useLoadUser();
  useAppHotkeys();

  const { space, loadingText, loadingProgress, join } = useSpace(id);

  useEffect(() => {
    if (!scriptsReady || !canvasRef.current || !overlayRef.current) return;

    const engine = new Engine({
      canvas: canvasRef.current,
      overlayCanvas: overlayRef.current,
    });

    engine.render.send({ subject: "set_animations_path", data: "/models" });
    engine.render.send({ subject: "set_default_avatar", data: "/models/Wired-chan.vrm" });
    engine.render.send({ subject: "set_skybox", data: { uri: "/images/Skybox.jpg" } });
    engine.physics.send({ subject: "start", data: null });

    useAppStore.setState({ engine });

    return () => {
      engine.destroy();
      useAppStore.setState({ engine: null, chatMessages: [] });
    };
  }, [scriptsReady]);

  useEffect(() => {
    if (!engine) return;
    join();
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

      <Script src="/scripts/draco_decoder.js" onReady={() => setScriptsReady(true)} />

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
        }}
      >
        {loaded && <Overlay />}

        <div className="h-full">
          <div ref={containerRef} className="relative h-full w-full overflow-hidden">
            <canvas ref={canvasRef} className={`h-full w-full transition ${loadedClass}`} />
            <canvas
              ref={overlayRef}
              className={`absolute top-0 left-0 z-10 h-full w-full transition ${loadedClass}`}
            />
          </div>
        </div>
      </div>
    </>
  );
}
