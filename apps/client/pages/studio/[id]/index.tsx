import { useEffect, useRef, useState } from "react";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";
import Split from "react-split";

import { Engine } from "@wired-xr/engine";

import StudioNavbar from "../../../src/studio/components/StudioNavbar/StudioNavbar";
import TreeMenu from "../../../src/studio/components/TreeMenu/TreeMenu";
import { useAutosave } from "../../../src/studio/hooks/useAutosave";
import { useFixSave } from "../../../src/studio/hooks/useFixSave";
import { useLoad } from "../../../src/studio/hooks/useLoad";
import { useStudioHotkeys } from "../../../src/studio/hooks/useStudioHotkeys";
import { useTransformControls } from "../../../src/studio/hooks/useTransformControls";
import { useStudioStore } from "../../../src/studio/store";
import MetaTags from "../../../src/ui/MetaTags";

export default function Studio() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [engineKey, setEngineKey] = useState(0);

  useFixSave();
  useLoad();
  useAutosave();
  useTransformControls();
  useStudioHotkeys();

  useEffect(() => {
    window.addEventListener("resize", updateCanvasSize);

    const canvas = canvasRef.current;
    if (!canvas) throw new Error("Canvas not found");

    // Set initial canvas size
    updateCanvasSize();

    // Create engine
    const engine = new Engine(canvas, {
      skyboxPath: "/images/skybox/",
      camera: "orbit",
      enableTransformControls: true,
    });

    useStudioStore.setState({ engine });

    return () => {
      engine.destroy();
      window.removeEventListener("resize", updateCanvasSize);
      useStudioStore.setState({ engine: null });
    };
  }, [engineKey]);

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
                  <div ref={containerRef} className="relative w-full h-full overflow-hidden">
                    <canvas key={engineKey} ref={canvasRef} className="w-full h-full" />
                  </div>
                </div>

                <div className="float-left h-full">{/* <InspectMenu /> */}</div>
              </Split>
            </div>

            <div>{/* <FileBrowser /> */}</div>
          </Split>
        </div>
      </DndProvider>
    </>
  );
}
