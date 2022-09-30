import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useMemo, useRef, useState } from "react";
import { MdClose } from "react-icons/md";

import { useLoad } from "../../../editor/hooks/useLoad";
import { useEditorStore } from "../../../editor/store";
import MetaTags from "../../../ui/MetaTags";

export default function Preview() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engine = useEditorStore((state) => state.engine);
  const [loaded, setLoaded] = useState(false);

  const router = useRouter();
  const id = router.query.id;

  useLoad();

  useEffect(() => {
    async function initEngine() {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error("Canvas not found");

      const { Engine } = await import("@wired-labs/engine");

      // Create engine
      const engine = new Engine({
        canvas,
        camera: "player",
        skyboxPath: "/images/skybox/",
      });

      useEditorStore.setState({ engine });

      // Start engine
      engine.start().then(() => {
        setLoaded(true);
      });
    }

    initEngine();
  }, [canvasRef]);

  useEffect(() => {
    if (!engine) return;

    return () => {
      engine.destroy();
      useEditorStore.setState({
        engine: null,
        selectedId: null,
      });
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

  const loadedClass = loaded ? "opacity-100" : "opacity-0";

  return (
    <>
      <MetaTags title="Preview" />

      <div className={`crosshair transition ${loadedClass}`} />

      <div className="h-full">
        <div
          ref={containerRef}
          className="relative h-full w-full overflow-hidden"
        >
          <canvas
            ref={canvasRef}
            className={`h-full w-full transition ${loadedClass}`}
          />
        </div>

        <div
          onClick={(e) => e.stopPropagation()}
          className="fixed top-0 right-0 p-6 text-2xl"
        >
          <Link href={`/editor/${id}`} passHref>
            <div className="block cursor-pointer rounded-full bg-surface/40 p-2 text-onSurface backdrop-blur transition hover:shadow active:opacity-80 active:shadow-sm">
              <MdClose />
            </div>
          </Link>
        </div>
      </div>
    </>
  );
}
