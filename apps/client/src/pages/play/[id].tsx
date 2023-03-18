import { Engine } from "engine";
import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";
import Script from "next/script";
import { useEffect, useRef, useState } from "react";

import MetaTags from "../../home/MetaTags";
import { useHotkeys } from "../../play/hooks/useHotkeys";
import { useLoadUser } from "../../play/hooks/useLoadUser";
import { useResizeCanvas } from "../../play/hooks/useResizeCanvas";
import { useSetAvatar } from "../../play/hooks/useSetAvatar";
import { useSpace } from "../../play/hooks/useSpace";
import { usePlayStore } from "../../play/store";
import LoadingScreen from "../../play/ui/LoadingScreen";
import Overlay from "../../play/ui/Overlay";
import { fetchSpaceMetadata } from "../../server/helpers/fetchSpaceMetadata";
import { toHex } from "../../utils/toHex";

export const getServerSideProps = async ({ res, query }: GetServerSidePropsContext) => {
  const ONE_MINUTE = 60;
  const ONE_MONTH = 60 * 60 * 24 * 30;

  res.setHeader(
    "Cache-Control",
    `public, max-age=0, s-maxage=${ONE_MINUTE}, stale-while-revalidate=${ONE_MONTH}`
  );

  const hexId = query.id as string;
  const id = parseInt(hexId);

  const metadata = await fetchSpaceMetadata(id);

  return {
    props: { id, metadata },
  };
};

type Props = InferGetServerSidePropsType<typeof getServerSideProps>;

export default function Play({ id, metadata }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);

  const [scriptsReady, setScriptsReady] = useState(false);
  const engine = usePlayStore((state) => state.engine);

  const setAvatar = useSetAvatar();
  useResizeCanvas(engine, canvasRef, overlayRef, containerRef);
  useLoadUser();
  useHotkeys();

  const { loadingText, loadingProgress, join } = useSpace(id, metadata);

  useEffect(() => {
    if (!scriptsReady || !canvasRef.current || !overlayRef.current) return;

    const engine = new Engine({
      canvas: canvasRef.current,
      overlayCanvas: overlayRef.current,
    });

    engine.render.send({ subject: "set_animations_path", data: "/models" });
    engine.render.send({ subject: "set_default_avatar", data: "/models/Wired-chan.vrm" });
    engine.render.send({ subject: "set_skybox", data: { uri: "/images/Skybox.jpg" } });
    engine.render.send({ subject: "toggle_animations", data: true });
    engine.physics.send({ subject: "start", data: null });

    usePlayStore.setState({ engine });

    return () => {
      engine.destroy();
      usePlayStore.setState({ engine: null, chatMessages: [] });
    };
  }, [scriptsReady]);

  useEffect(() => {
    if (!engine) return;

    join();

    return () => {
      engine.scene.clear();
    };
  }, [engine, join]);

  const loaded = loadingProgress === 1;
  const loadedClass = loaded ? "opacity-100" : "opacity-0";

  return (
    <>
      <MetaTags
        title={metadata?.name ?? `Space ${toHex(id)}`}
        description={metadata?.description ?? ""}
        image={metadata?.image ?? ""}
      />

      <Script src="/scripts/draco_decoder.js" onReady={() => setScriptsReady(true)} />

      <LoadingScreen
        text={metadata?.name}
        image={metadata?.image}
        loadingProgress={loadingProgress}
        loadingText={loadingText}
      />

      <div
        className="h-screen w-screen"
        onDragOver={(e) => e.preventDefault()}
        onDrop={(e) => {
          const { engine } = usePlayStore.getState();
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
        {loaded && <Overlay id={id} />}

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
