import { Engine } from "engine";
import { useCallback } from "react";

export function useResizeCanvas(engine: Engine | null) {
  const resize = useCallback(
    (width: number, height: number) => {
      if (!engine) return;

      if (engine.overlayCanvas) {
        engine.overlayCanvas.width = width;
        engine.overlayCanvas.height = height;
      }

      if (engine.canvas instanceof HTMLCanvasElement) {
        engine.canvas.width = width;
        engine.canvas.height = height;
      }

      engine.render.send({ data: { height, width }, subject: "set_size" });
    },
    [engine]
  );

  return resize;
}
