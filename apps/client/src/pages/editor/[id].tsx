import { Engine } from "engine";
import Script from "next/script";
import { useEffect, useRef } from "react";
import Split from "react-split";

import { useResizeCanvas } from "../../app/hooks/useResizeCanvas";
import EditorNavbar from "../../editor/components/EditorNavbar/EditorNavbar";
import InspectMenu from "../../editor/components/InspectMenu/InspectMenu";
import TreeMenu from "../../editor/components/TreeMenu/TreeMenu";
import { useAutosave } from "../../editor/hooks/useAutosave";
import { useEditorHotkeys } from "../../editor/hooks/useEditorHotkeys";
import { useLoad } from "../../editor/hooks/useLoad";
import { useTransformControls } from "../../editor/hooks/useTransformControls";
import { useEditorStore } from "../../editor/store";
import MetaTags from "../../home/MetaTags";
import Spinner from "../../ui/Spinner";

export default function Editor() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);

  const engine = useEditorStore((state) => state.engine);
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);

  const resize = useResizeCanvas(engine, canvasRef, overlayRef, containerRef);
  useLoad();
  useAutosave();
  useTransformControls();
  useEditorHotkeys();

  useEffect(() => {
    if (!engine) return;

    return () => {
      engine.destroy();
      useEditorStore.setState({
        engine: null,
        canvas: null,
        selectedId: null,
        treeIds: [],
        openIds: [],
      });
    };
  }, [engine]);

  const loadedClass = sceneLoaded ? "opacity-100" : "opacity-0";

  return (
    <>
      <MetaTags title="Editor" />

      <Script
        src="/scripts/draco_decoder.js"
        onReady={() => {
          if (!canvasRef.current || !overlayRef.current) throw new Error("Canvas not found");

          const engine = new Engine({
            canvas: canvasRef.current,
            overlayCanvas: overlayRef.current,
          });
          engine.controls = "orbit";
          engine.visuals = true;

          engine.render.send({ subject: "set_animations_path", data: "/models" });
          engine.render.send({ subject: "set_default_avatar", data: "/models/Wired-chan.vrm" });
          engine.render.send({ subject: "set_skybox", data: { uri: "/images/Skybox.jpg" } });

          useEditorStore.setState({ engine, canvas: canvasRef.current, visuals: true });
        }}
      />

      <div
        className="absolute top-0 left-0 h-full w-full overflow-hidden"
        onDragOver={(e) => e.preventDefault()}
        onDrop={async (e) => {
          if (!engine) return;

          e.preventDefault();

          const item = e.dataTransfer.items[0];
          if (item?.kind !== "file") return;

          const file = item.getAsFile();
          if (!file) return;

          const isGLTF = file.name.endsWith(".gltf") || file.name.endsWith(".glb");
          if (!isGLTF) return;

          await engine.scene.addFile(file);
        }}
      >
        <div className="z-10 h-14 w-full border-b">
          <EditorNavbar />
        </div>

        <Split
          sizes={[15, 65, 20]}
          minSize={[50, 400, 50]}
          direction="horizontal"
          gutterSize={4}
          className="flex h-full"
          onMouseUp={resize}
        >
          <div>
            <TreeMenu />
          </div>

          <div className="border-x">
            <div ref={containerRef} className="relative h-full w-full overflow-hidden">
              {!sceneLoaded && (
                <div className="absolute top-0 left-0 flex h-full w-full items-center justify-center">
                  <div className="flex h-full  flex-col items-center justify-center">
                    <Spinner />
                  </div>
                </div>
              )}

              <canvas ref={canvasRef} className={`h-full w-full transition ${loadedClass}`} />
              <canvas
                ref={overlayRef}
                className={`absolute top-0 left-0 z-10 h-full w-full transition ${loadedClass}`}
              />
            </div>
          </div>

          <div>
            <InspectMenu />
          </div>
        </Split>
      </div>
    </>
  );
}
