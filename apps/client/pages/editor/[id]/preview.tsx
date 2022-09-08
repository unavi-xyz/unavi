import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useRef } from "react";
import { MdClose } from "react-icons/md";

import { Engine } from "@wired-labs/engine";

import { useLoad } from "../../../src/editor/hooks/useLoad";
import { useEditorStore } from "../../../src/editor/store";
import MetaTags from "../../../src/ui/MetaTags";

export default function Preview() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const router = useRouter();
  const id = router.query.id;

  useLoad();

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    // Set initial canvas size
    updateCanvasSize();
    window.addEventListener("resize", updateCanvasSize);

    // Start engine
    const engine = new Engine(canvas, {
      skyboxPath: "/images/skybox/",
      camera: "player",
    });
    useEditorStore.setState({ engine });

    return () => {
      engine.destroy();
      useEditorStore.setState({ engine: null });
      window.removeEventListener("resize", updateCanvasSize);
    };
  }, [canvasRef]);

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
    <>
      <MetaTags title="Preview" />

      <div className="crosshair" />

      <div className="h-full">
        <div
          ref={containerRef}
          className="relative w-full h-full overflow-hidden"
        >
          <canvas ref={canvasRef} className="w-full h-full" />
        </div>

        <div
          onClick={(e) => e.stopPropagation()}
          className="fixed top-0 right-0 p-6 text-2xl"
        >
          <Link href={`/editor/${id}`} passHref>
            <a
              className="block cursor-pointer p-2 rounded-full bg-surface text-onSurface
                         backdrop-blur bg-opacity-60 hover:bg-opacity-100 transition active:bg-opacity-90"
            >
              <MdClose />
            </a>
          </Link>
        </div>
      </div>
    </>
  );
}
