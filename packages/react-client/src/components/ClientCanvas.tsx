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
        height: "100%",
        overflow: "hidden",
        position: "relative",
        width: "100%",
      }}
    >
      <canvas
        ref={canvasRef}
        style={{
          height: "100%",
          width: "100%",
        }}
      />
      <canvas
        ref={overlayRef}
        style={{
          height: "100%",
          inset: 0,
          position: "absolute",
          width: "100%",
          zIndex: 10,
        }}
      />
    </div>
  );
}
