import { Engine } from "engine";
import { useCallback, useEffect, useRef } from "react";

import { useResizeCanvas } from "../hooks/useResizeCanvas";

interface Props {
  engine: Engine | null;
}

export default function ClientCanvas({ engine }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const resize = useResizeCanvas(engine);
  const onResize = useCallback(() => {
    if (!containerRef.current) return;
    resize(containerRef.current.clientWidth, containerRef.current.clientHeight);
  }, [resize]);

  // Set engine canvas
  useEffect(() => {
    if (!engine) return;
    engine.canvas = canvasRef.current;
    engine.overlayCanvas = overlayRef.current;
  }, [engine]);

  // Resize canvas on window resize
  useEffect(() => {
    onResize();

    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, [onResize]);

  return (
    <div
      ref={containerRef}
      style={{
        position: "relative",
        width: "100%",
        height: "100%",
        overflow: "hidden",
      }}
    >
      <canvas
        ref={canvasRef}
        style={{
          width: "100%",
          height: "100%",
        }}
      />
      <canvas
        ref={overlayRef}
        style={{
          position: "absolute",
          inset: 0,
          zIndex: 10,
          width: "100%",
          height: "100%",
        }}
      />
    </div>
  );
}
