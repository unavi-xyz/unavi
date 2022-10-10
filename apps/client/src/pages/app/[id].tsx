import { Entity, GLTFMesh } from "@wired-labs/engine";
import { NextPageContext } from "next";
import { useEffect, useMemo, useRef, useState } from "react";

import ChatBox from "../../app/ChatBox";
import { useAppStore } from "../../app/store";
import { useAppHotkeys } from "../../app/useAppHotkeys";
import {
  getPublicationProps,
  PublicationProps,
} from "../../client/lens/utils/getPublicationProps";
import MetaTags from "../../home/MetaTags";
import Spinner from "../../ui/Spinner";

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
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const createdEngine = useRef(false);
  const [engineStarted, setEngineStarted] = useState(false);

  const engine = useAppStore((state) => state.engine);

  const modelURL: string | undefined =
    publication?.metadata.media[1]?.original.url;

  useAppHotkeys();

  useEffect(() => {
    if (!engine) return;

    engine.joinSpace(id).then(() => {
      // Start engine
      engine.start();
      setEngineStarted(true);
    });

    return () => {
      engine.leaveSpace();
    };
  }, [engine, id]);

  useEffect(() => {
    if (!modelURL || !engine) return;

    // Create glTF entity
    const entity = new Entity();
    const mesh = new GLTFMesh();
    mesh.uri = modelURL;
    entity.mesh = mesh;

    // Add entity to scene
    engine.scene.addEntity(entity);

    return () => {
      engine.scene.removeEntity(entity.id);
    };
  }, [engine, modelURL]);

  useEffect(() => {
    if (createdEngine.current) return;
    createdEngine.current = true;

    async function initEngine() {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error("Canvas not found");

      const { Engine } = await import("@wired-labs/engine");

      // Create engine
      const engine = new Engine({
        canvas,
        camera: "player",
        skyboxPath: "/images/skybox/",
        avatarPath: "/models/avatar.vrm",
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
      if (process.env.NODE_ENV === "development") window.location.reload();
    };
  }, [engine]);

  const updateCanvasSize = useMemo(() => {
    return () => {
      if (typeof OffscreenCanvas !== "undefined") {
        if (!engine) return;
        const resize = engine.renderThread.onResize.bind(engine.renderThread);
        resize();
        return;
      }

      try {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const container = containerRef.current;
        if (!container) return;

        // Resize canvas
        canvas.width = container.clientWidth;
        canvas.height = container.clientHeight;
      } catch (e) {
        console.error(e);
      }
    };
  }, [engine]);

  useEffect(() => {
    // Set initial canvas size
    updateCanvasSize();

    window.addEventListener("resize", updateCanvasSize);
    return () => {
      window.removeEventListener("resize", updateCanvasSize);
    };
  }, [updateCanvasSize]);

  const loadedClass = engineStarted ? "opacity-100" : "opacity-0";

  return (
    <>
      <MetaTags
        title={metadata.title ?? id}
        description={metadata.description ?? undefined}
        image={metadata.image ?? undefined}
        card="summary_large_image"
      />

      <div className="h-full">
        {engineStarted ? (
          <div className="crosshair" />
        ) : (
          <div className="absolute top-0 left-0 flex h-full w-full items-center justify-center">
            <div className="flex h-full flex-col items-center justify-center">
              <Spinner />
            </div>
          </div>
        )}

        <div
          ref={containerRef}
          className="relative h-full w-full overflow-hidden"
        >
          <canvas
            ref={canvasRef}
            className={`h-full w-full transition ${loadedClass}`}
          />
        </div>
      </div>

      <div className="absolute left-0 bottom-0 m-4">
        <ChatBox />
      </div>
    </>
  );
}
