import { Entity, GLTFMesh } from "@wired-labs/engine";
import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useMemo, useRef, useState } from "react";
import { MdClose } from "react-icons/md";

import { useEditorStore } from "../../../editor/store";
import MetaTags from "../../../ui/MetaTags";

export default function Preview() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engine = useEditorStore((state) => state.engine);
  const exportedScene = useEditorStore((state) => state.exportedScene);
  const [loaded, setLoaded] = useState(false);

  const router = useRouter();
  const id = router.query.id;

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

  useEffect(() => {
    if (!engine || !exportedScene || !loaded) return;

    // Load exported scene
    const blob = new Blob([exportedScene], { type: "model/gltf-binary" });
    const url = URL.createObjectURL(blob);

    // Create glTF entity
    const entity = new Entity();
    const mesh = new GLTFMesh();
    mesh.uri = url;
    entity.mesh = mesh;

    // Add entity to scene
    engine.scene.addEntity(entity);

    return () => {
      engine.scene.removeEntity(entity.id);
    };
  }, [engine, exportedScene, loaded]);

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
