import { Engine } from "engine";
import { useEffect, useMemo, useState } from "react";

export function useResizeEngineCanvas(
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

    engine.waitForReady().then(() => {
      if (engine.input) setInputCanvas(engine.input.canvas);
    });
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

  const updateCanvasSize = useMemo(() => {
    return () => {
      if (inputCanvas) {
        inputCanvas.width = window.innerWidth;
        inputCanvas.height = window.innerHeight;
      }

      if (typeof OffscreenCanvas !== "undefined") {
        if (!engine) return;
        const resize = engine.renderThread.onResize.bind(engine.renderThread);
        resize();
      } else {
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
    };
  }, [canvasRef, containerRef, engine, inputCanvas]);

  useEffect(() => {
    // Set initial canvas size
    updateCanvasSize();

    window.addEventListener("resize", updateCanvasSize);
    return () => {
      window.removeEventListener("resize", updateCanvasSize);
    };
  }, [updateCanvasSize]);

  return updateCanvasSize;
}
