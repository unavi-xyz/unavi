import { Engine } from "engine";
import { useEffect, useMemo } from "react";

export function useResizeCanvas(
  engine: Engine | null,
  canvasRef: React.RefObject<HTMLCanvasElement>,
  overlayRef: React.RefObject<HTMLCanvasElement>,
  containerRef: React.RefObject<HTMLDivElement>
) {
  const resize = useMemo(() => {
    return () => {
      if (!engine) return;

      const canvas = canvasRef.current;
      if (!canvas) return;

      const overlay = overlayRef.current;
      if (!overlay) return;

      const container = containerRef.current;
      if (!container) return;

      overlay.width = container.clientWidth;
      overlay.height = container.clientHeight;

      if (typeof OffscreenCanvas === "undefined") {
        canvas.width = container.clientWidth;
        canvas.height = container.clientHeight;
      }

      engine.render.send({
        subject: "set_size",
        data: { width: canvas.clientWidth, height: canvas.clientHeight },
      });
    };
  }, [canvasRef, overlayRef, containerRef, engine]);

  useEffect(() => {
    // Set initial canvas size
    resize();

    window.addEventListener("resize", resize);
    return () => window.removeEventListener("resize", resize);
  }, [resize]);

  return resize;
}
