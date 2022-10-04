import { Entity, GLTFMesh } from "@wired-labs/engine";
import { useEffect, useRef, useState } from "react";

import { useStore } from "./store";

export interface RenderInfo {
  load: {
    time: number;
    threeTime?: number;
    exportedTime?: number;
  };
  memory: {
    geometries: number;
    textures: number;
  };
  render: {
    calls: number;
    lines: number;
    points: number;
    triangles: number;
  };
}

export interface Settings {
  testThree: boolean;
  testExport: boolean;
}

export default function ExampleCanvas() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const initialized = useRef(false);

  const engine = useStore((state) => state.engine);
  const uri = useStore((state) => state.uri);

  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || initialized.current) return;

    initialized.current = true;

    // Set canvas size
    updateCanvasSize();
    window.addEventListener("resize", updateCanvasSize);

    async function initEngine() {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error("Canvas not found");

      const { Engine } = await import("@wired-labs/engine");

      // Create engine
      const newEngine = new Engine({
        canvas,
        camera: "orbit",
        skyboxPath: "/images/skybox/",
      });

      useStore.setState({ engine: newEngine });

      // Start engine
      newEngine.start().then(() => {
        setLoaded(true);
      });
    }

    initEngine();
  }, [canvasRef, engine]);

  useEffect(() => {
    if (!engine) return;

    return () => {
      engine.destroy();
      window.removeEventListener("resize", updateCanvasSize);
    };
  }, [engine]);

  useEffect(() => {
    if (!engine || !uri) return;

    // Create glTF entity
    const entity = new Entity();
    const mesh = new GLTFMesh();
    mesh.uri = uri;
    entity.mesh = mesh;

    // Add entity to scene
    engine.scene.addEntity(entity);

    return () => {
      engine.scene.removeEntity(entity.id);
    };
  }, [engine, uri]);

  const loadedClass = loaded ? "opacity-100" : "opacity-0";

  function updateCanvasSize() {
    // Only run if canvas is available
    // (canvas is not available when using OffscreenCanvas)
    if (typeof OffscreenCanvas !== "undefined" && initialized.current) return;

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
  }

  return (
    <div ref={containerRef} className="relative h-full w-full overflow-hidden">
      <canvas ref={canvasRef} className={`h-full w-full ${loadedClass}`} />
    </div>
  );
}
