import { useEffect, useMemo, useRef, useState } from "react";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";
import Split from "react-split";

import EditorNavbar from "../../../src/editor/components/EditorNavbar/EditorNavbar";
import InspectMenu from "../../../src/editor/components/InspectMenu/InspectMenu";
import TreeMenu from "../../../src/editor/components/TreeMenu/TreeMenu";
import { emptyScene } from "../../../src/editor/constants";
import { useAutosave } from "../../../src/editor/hooks/useAutosave";
import { useEditorHotkeys } from "../../../src/editor/hooks/useEditorHotkeys";
import { useLoad } from "../../../src/editor/hooks/useLoad";
import { useTransformControls } from "../../../src/editor/hooks/useTransformControls";
import { useEditorStore } from "../../../src/editor/store";
import MetaTags from "../../../src/ui/MetaTags";
import { deepClone } from "../../../src/utils/deepClone";

export default function Editor() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const engine = useEditorStore((state) => state.engine);

  const [loaded, setLoaded] = useState(false);

  useLoad();
  useAutosave();
  // useTransformControls();
  // useEditorHotkeys();

  useEffect(() => {
    async function initEngine() {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error("Canvas not found");

      const { Engine } = await import("@wired-labs/engine");

      // Create engine
      const engine = new Engine({
        canvas,
        skyboxPath: "/images/skybox/",
        camera: "orbit",
        enableTransformControls: true,
        preserveDrawingBuffer: true,
      });

      useEditorStore.setState({ engine });

      // Start engine
      // engine.start().then(() => {
      setLoaded(true);
      // });
    }

    initEngine();
  }, []);

  useEffect(() => {
    if (!engine) return;

    return () => {
      engine.destroy();
      useEditorStore.setState({
        engine: null,
        selectedId: null,
      });
    };
  }, [engine]);

  const updateCanvasSize = useMemo(() => {
    return () => {
      if (typeof OffscreenCanvas !== "undefined") {
        if (!engine) return;
        // const resize = engine.renderThread.onResize.bind(engine.renderThread);
        // resize();
        return;
      }

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
    };
  }, [engine]);

  useEffect(() => {
    // Set initial canvas size
    updateCanvasSize();

    window.addEventListener("resize", updateCanvasSize);
    return () => {
      window.removeEventListener("resize", updateCanvasSize);
    };
  }, [updateCanvasSize]);

  const loadedClass = loaded ? "opacity-100" : "opacity-0";

  return (
    <>
      <MetaTags title="Editor" />

      <DndProvider backend={HTML5Backend}>
        <div className="flex h-full flex-col overflow-hidden">
          <div className="z-10 h-14 w-full border-b">
            <EditorNavbar />
          </div>

          <Split
            sizes={[5, 85, 10]}
            minSize={[250, 300, 430]}
            direction="horizontal"
            expandToMin
            gutterSize={6}
            className="h-full"
            onMouseUp={updateCanvasSize}
          >
            <div className="float-left h-full">
              <TreeMenu />
            </div>

            <div className="float-left h-full border-x">
              <div
                ref={containerRef}
                className="relative h-full w-full overflow-hidden"
              >
                <canvas
                  ref={canvasRef}
                  className={`h-full w-full transition ${loadedClass}`}
                />
              </div>
            </div>

            <div className="float-left h-full">
              <InspectMenu />
            </div>
          </Split>
        </div>
      </DndProvider>
    </>
  );
}
//
