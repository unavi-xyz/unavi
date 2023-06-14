import { useEffect, useRef } from "react";

/**
 * An auto-resizing canvas component.
 */
export default function Canvas() {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    function resize() {
      if (!canvas.current) return;
      canvas.current.width = window.innerWidth;
      canvas.current.height = window.innerHeight;
    }

    resize();

    window.addEventListener("resize", resize);

    return () => {
      window.removeEventListener("resize", resize);
    };
  }, [canvas]);

  return <canvas ref={canvas} style={{ flex: 1, touchAction: "none", width: "100%" }} />;
}
