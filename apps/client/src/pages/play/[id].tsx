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
import { toHex } from "../../utils/toHex";

export const getServerSideProps = async ({ query }: GetServerSidePropsContext) => {
  const hexId = query.id as string;
  const id = parseInt(hexId);

  return {
    props: { id },
  };
};

export default function Play({ id }: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);

  const [scriptsReady, setScriptsReady] = useState(false);
  const engine = usePlayStore((state) => state.engine);

  const setAvatar = useSetAvatar();
  useResizeCanvas(engine, canvasRef, overlayRef, containerRef);
  useLoadUser();
  useHotkeys();

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

    usePlayStore.setState({ engine });

    return () => {
      engine.destroy();
      usePlayStore.setState({ engine: null, chatMessages: [] });
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
        title={space?.metadata?.name ?? `Space ${toHex(id)}`}
        description={space?.metadata?.description ?? ""}
        image={space?.metadata?.image ?? ""}
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
