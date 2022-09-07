import { useEffect, useRef } from "react";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";
import Split from "react-split";

import InspectMenu from "../../../src/studio/components/InspectMenu/InspectMenu";
import StudioNavbar from "../../../src/studio/components/StudioNavbar/StudioNavbar";
import TreeMenu from "../../../src/studio/components/TreeMenu/TreeMenu";
import { emptyTree } from "../../../src/studio/constants";
import { useLoad } from "../../../src/studio/hooks/useLoad";
import { useStudioHotkeys } from "../../../src/studio/hooks/useStudioHotkeys";
import { useTransformControls } from "../../../src/studio/hooks/useTransformControls";
import { useStudioStore } from "../../../src/studio/store";
import MetaTags from "../../../src/ui/MetaTags";
import { deepClone } from "../../../src/utils/deepClone";

export default function Studio() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const engine = useStudioStore((state) => state.engine);

  useLoad();
  // useAutosave();
  useTransformControls();
  useStudioHotkeys();

  useEffect(() => {
    async function initEngine() {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error("Canvas not found");

      const { Engine } = await import("@wired-labs/engine");

      // Create engine
      const engine = new Engine(canvas, {
        skyboxPath: "/images/skybox/",
        camera: "orbit",
        enableTransformControls: true,
      });

      useStudioStore.setState({ engine });
    }

    initEngine();
  }, []);

  useEffect(() => {
    if (!engine) return;

    return () => {
      engine.destroy();
      useStudioStore.setState({ engine: null, tree: deepClone(emptyTree) });
    };
  }, [engine]);

  useEffect(() => {
    // Set initial canvas size
    updateCanvasSize();

    window.addEventListener("resize", updateCanvasSize);
    return () => {
      window.removeEventListener("resize", updateCanvasSize);
    };
  }, []);

  function updateCanvasSize() {
    // Only run if canvas is available
    // (canvas is not available when using OffscreenCanvas)
    if (typeof OffscreenCanvas !== "undefined") return;

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

  return (
    <>
      <MetaTags title="Studio" />

      <DndProvider backend={HTML5Backend}>
        <div className="h-full overflow-hidden flex flex-col">
          <div className="border-b w-full h-14 z-10">
            <StudioNavbar />
          </div>

          <Split
            sizes={[70, 30]}
            minSize={200}
            direction="vertical"
            expandToMin
            gutterSize={6}
            className="h-full"
            onDrag={updateCanvasSize}
          >
            <div className="h-full">
              <Split
                sizes={[15, 65, 20]}
                minSize={200}
                direction="horizontal"
                expandToMin
                gutterSize={6}
                className="h-full"
                onDrag={updateCanvasSize}
              >
                <div className="float-left h-full">
                  <TreeMenu />
                </div>

                <div className="float-left h-full border-x">
                  <div
                    ref={containerRef}
                    className="relative w-full h-full overflow-hidden"
                  >
                    <canvas ref={canvasRef} className="w-full h-full" />
                  </div>
                </div>

                <div className="float-left h-full">
                  <InspectMenu />
                </div>
              </Split>
            </div>

            <div>{/* <FileBrowser /> */}</div>
          </Split>
        </div>
      </DndProvider>
    </>
  );
}
//
