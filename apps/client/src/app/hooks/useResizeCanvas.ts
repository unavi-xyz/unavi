import { Engine } from "engine";
import { useEffect, useMemo, useState } from "react";

export function useResizeCanvas(
  engine: Engine | null,
  canvasRef: React.RefObject<HTMLCanvasElement>,
  containerRef: React.RefObject<HTMLDivElement>
) {
  const [inputCanvas, setInputCanvas] = useState<HTMLCanvasElement | null>(null);

  useEffect(() => {
    if (!engine) {
      setInputCanvas(null);
      return;
    }

    // engine.waitForReady().then(() => {
    //   if (engine.input) setInputCanvas(engine.input.canvas);
    // });
  }, [engine]);

  useEffect(() => {
    if (!inputCanvas) return;

    const container = containerRef.current;
    if (!container) return;

    container.appendChild(inputCanvas);

    inputCanvas.style.position = "absolute";
    inputCanvas.style.top = "0";
    inputCanvas.style.left = "0";
    inputCanvas.style.width = "100%";
    inputCanvas.style.height = "100%";
    inputCanvas.style.zIndex = "5";

    return () => {
      inputCanvas.remove();
    };
  }, [inputCanvas, containerRef]);

  const resize = useMemo(() => {
    return () => {
      if (!engine) return;

      if (inputCanvas) {
        inputCanvas.width = window.innerWidth;
        inputCanvas.height = window.innerHeight;
      }

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
  }, [canvasRef, containerRef, engine, inputCanvas]);

  useEffect(() => {
    // Set initial canvas size
    resize();

    window.addEventListener("resize", resize);
    return () => {
      window.removeEventListener("resize", resize);
    };
  }, [resize]);

  return resize;
}
