import { Engine } from "engine";
import { useEffect, useMemo } from "react";

export function useResizeCanvas(
  engine: Engine | null,
  canvasRef: React.RefObject<HTMLCanvasElement>,
  containerRef: React.RefObject<HTMLDivElement>
) {
  const resize = useMemo(() => {
    return () => {
      if (!engine) return;

      const canvas = canvasRef.current;
      if (!canvas) return;

      const container = containerRef.current;
      if (!container) return;

      if (typeof OffscreenCanvas === "undefined") {
        canvas.width = container.clientWidth;
        canvas.height = container.clientHeight;
      }

      engine.modules.render.toRenderThread({
        subject: "set_size",
        data: { width: canvas.clientWidth, height: canvas.clientHeight },
      });
    };
  }, [canvasRef, containerRef, engine]);

  useEffect(() => {
    // Set initial canvas size
    resize();

    window.addEventListener("resize", resize);
    return () => window.removeEventListener("resize", resize);
  }, [resize]);

  return resize;
}
