import { useEffect, useRef, useState } from "react";

import { Engine } from "@wired-xr/engine";

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

interface Props {
  uri: string;
}

export default function ExampleCanvas({ uri }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [engine, setEngine] = useState<Engine>();
  const initialized = useRef(false);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || initialized.current) return;

    initialized.current = true;

    // Set canvas size
    updateCanvasSize();
    window.addEventListener("resize", updateCanvasSize);

    // Create engine
    const newEngine = new Engine(canvas, { controls: "orbit" });

    if (engine) {
      engine.destroy();
      window.removeEventListener("resize", updateCanvasSize);
    }

    setEngine(newEngine);
  }, [canvasRef, engine, uri]);

  useEffect(() => {
    if (!engine) return;
    // Load gltf
    engine.loadGltf(uri);
  }, [engine, uri]);

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
    <div ref={containerRef} className="relative w-full h-full overflow-hidden">
      <canvas ref={canvasRef} className="w-full h-full" />
    </div>
  );
}
