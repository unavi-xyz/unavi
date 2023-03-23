"use client";

import { ERC721Metadata } from "contracts";
import { Engine } from "engine";
import Script from "next/script";
import { useEffect, useRef, useState } from "react";

import { useAvatarEquip } from "../../../src/play/hooks/useAvatarEquip";
import { useHotkeys } from "../../../src/play/hooks/useHotkeys";
import { useLoadUser } from "../../../src/play/hooks/useLoadUser";
import { useResizeCanvas } from "../../../src/play/hooks/useResizeCanvas";
import { useSetAvatar } from "../../../src/play/hooks/useSetAvatar";
import { useSpace } from "../../../src/play/hooks/useSpace";
import LoadingScreen from "../../../src/play/ui/LoadingScreen";
import Overlay from "../../../src/play/ui/Overlay";
import { usePlayStore } from "./store";

interface Props {
  id: number;
  metadata: ERC721Metadata;
}

export default function App({ id, metadata }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);

  const [scriptsReady, setScriptsReady] = useState(false);
  const engine = usePlayStore((state) => state.engine);
  const avatar = usePlayStore((state) => state.avatar);

  const setAvatar = useSetAvatar();
  const equipAction = useAvatarEquip(engine, avatar, setAvatar);
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

  useEffect(() => {
    if (!engine) return;

    // Attach engine to window for debugging
    (window as any).engine = engine;

    return () => {
      delete (window as any).engine;
    };
  }, [engine, join]);

  const loaded = loadingProgress === 1;
  const loadedClass = loaded ? "opacity-100" : "opacity-0";

  return (
    <>
      <Script src="/scripts/draco_decoder.js" onReady={() => setScriptsReady(true)} />

      <LoadingScreen
        text={metadata?.name}
        image={metadata?.image}
        loadingProgress={loadingProgress}
        loadingText={loadingText}
      />

      <div
        className="fixed h-screen w-screen"
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
        {loaded && <Overlay id={id} action={equipAction} />}

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
