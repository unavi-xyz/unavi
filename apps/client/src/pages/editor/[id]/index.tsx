import { Engine } from "engine";
import Script from "next/script";
import { useEffect, useRef } from "react";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";

import { useResizeCanvas } from "../../../app/hooks/useResizeCanvas";
import { useEditorStore } from "../../../editor/store";
import MetaTags from "../../../home/MetaTags";

export default function Editor() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const createdEngine = useRef(false);

  const engine = useEditorStore((state) => state.engine);
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);

  useResizeCanvas(engine, canvasRef, containerRef);
  // useLoad();
  // useAutosave();
  // useTransformControls();
  // useEditorHotkeys();

  useEffect(() => {
    if (!canvasRef.current || createdEngine.current) return;

    const engine = new Engine({ canvas: canvasRef.current });
    createdEngine.current = true;

    useEditorStore.setState({ engine, sceneLoaded: true });
  }, []);

  const loadedClass = sceneLoaded ? "opacity-100" : "opacity-0";

  return (
    <>
      <MetaTags title="Editor" />

      <Script src="/scripts/draco_encoder.js" />
      <Script src="/scripts/draco_decoder.js" />

      <DndProvider backend={HTML5Backend}>
        <div
          className="flex h-screen flex-col overflow-hidden"
          onDragOver={(e) => e.preventDefault()}
          onDrop={(e) => {
            const { engine } = useEditorStore.getState();
            if (!engine) return;

            e.preventDefault();

            const item = e.dataTransfer.items[0];
            if (item?.kind !== "file") return;

            const file = item.getAsFile();
            if (!file) return;

            const isGLTF = file.name.endsWith(".gltf") || file.name.endsWith(".glb");
            if (!isGLTF) return;
          }}
        >
          <div className="z-10 h-14 w-full border-b">{/* <EditorNavbar /> */}</div>

          <div className="h-full border-x">
            <div ref={containerRef} className="relative h-full w-full overflow-hidden">
              {/* {!sceneLoaded && (
                  <div className="absolute top-0 left-0 flex h-full w-full items-center justify-center">
                    <div className="flex h-full  flex-col items-center justify-center">
                      <Spinner />
                    </div>
                  </div>
                )} */}

              <canvas ref={canvasRef} className={`h-full w-full transition ${loadedClass}`} />
            </div>
          </div>

          {/* <Split
            sizes={[15, 65, 20]}
            minSize={[250, 500, 400]}
            direction="horizontal"
            expandToMin
            gutterSize={6}
            className="flex h-full"
            onMouseMove={resize}
          >
            <TreeMenu />

            <div className="h-full"><InspectMenu /></div>
          </Split> */}
        </div>
      </DndProvider>
    </>
  );
}
