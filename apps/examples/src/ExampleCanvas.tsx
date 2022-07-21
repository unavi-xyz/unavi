import { useEffect, useRef } from "react";

import { Engine } from "@wired-xr/new-engine";

interface Props {
  src: string;
}

export default function ExampleCanvas({ src }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvasRef.current) return;

    // Set canvas size
    updateCanvasSize();
    window.addEventListener("resize", updateCanvasSize);

    // Create engine
    const engine = new Engine({ canvas: canvasRef.current });

    // Move stats to the right
    const stats = document.getElementById("stats");
    if (stats) {
      stats.style.left = "calc(100% - 79px)";
    }

    // Load gltf
    engine.loadGltf(src);

    return () => {
      engine.destroy();
      window.removeEventListener("resize", updateCanvasSize);
    };
  }, [canvasRef, src]);

  function updateCanvasSize() {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const container = containerRef.current;
    if (!container) return;

    // Resize canvas
    canvas.width = container.clientWidth;
    canvas.height = container.clientHeight;
  }

  return (
    <div ref={containerRef} className="relative w-full h-full overflow-hidden">
      <canvas ref={canvasRef} className="w-full h-full" />
    </div>
  );
}
