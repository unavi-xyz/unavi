import { AutoCollider, GLTFMesh, Node } from "engine";
import Script from "next/script";
import { useEffect, useMemo, useRef } from "react";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";
import Split from "react-split";

import { addMesh } from "../../../editor/actions/AddMeshAction";
import { addNode } from "../../../editor/actions/AddNodeAction";
import EditorNavbar from "../../../editor/components/EditorNavbar/EditorNavbar";
import InspectMenu from "../../../editor/components/InspectMenu/InspectMenu";
import TreeMenu from "../../../editor/components/TreeMenu/TreeMenu";
import { useAutosave } from "../../../editor/hooks/useAutosave";
import { useEditorHotkeys } from "../../../editor/hooks/useEditorHotkeys";
import { useLoad } from "../../../editor/hooks/useLoad";
import { useTransformControls } from "../../../editor/hooks/useTransformControls";
import { useEditorStore } from "../../../editor/store";
import { updateGltfColliders } from "../../../editor/utils/updateGltfColliders";
import MetaTags from "../../../home/MetaTags";
import Spinner from "../../../ui/Spinner";

export default function Editor() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const createdEngine = useRef(false);

  const engine = useEditorStore((state) => state.engine);
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);

  useLoad();
  useAutosave();
  useTransformControls();
  useEditorHotkeys();

  useEffect(() => {
    if (createdEngine.current) return;
    createdEngine.current = true;

    async function initEngine() {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error("Canvas not found");

      useEditorStore.setState({ sceneLoaded: false });

      const { Engine } = await import("engine");

      // Create engine
      const engine = new Engine({
        canvas,
        camera: "orbit",
        enableTransformControls: true,
        preserveDrawingBuffer: true,
        skyboxPath: "/images/skybox/",
      });

      await engine.waitForReady();

      engine.renderThread.postMessage({
        subject: "show_visuals",
        data: { visible: useEditorStore.getState().visuals },
      });

      useEditorStore.setState({ engine, canvas });
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
        const resize = engine.renderThread.onResize.bind(engine.renderThread);
        resize();
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

            // Create mesh
            const mesh = new GLTFMesh();
            mesh.name = file.name;
            mesh.uri = URL.createObjectURL(file);

            // Create node
            const node = new Node();
            node.meshId = mesh.id;
            node.collider = new AutoCollider();

            const name = file.name.split(".").slice(0, -1).join(".");
            node.name = name;

            // Add node to scene
            addMesh(mesh);
            addNode(node);

            // Select node
            useEditorStore.setState({ selectedId: node.id });

            setTimeout(() => updateGltfColliders(node.id), 5000);
          }}
        >
          <div className="z-10 h-14 w-full border-b">
            <EditorNavbar />
          </div>

          <Split
            sizes={[15, 65, 20]}
            minSize={[250, 500, 400]}
            direction="horizontal"
            expandToMin
            gutterSize={6}
            className="flex h-full"
            onMouseUp={updateCanvasSize}
          >
            <TreeMenu />

            <div className="h-full border-x">
              <div ref={containerRef} className="relative h-full w-full overflow-hidden">
                {!sceneLoaded && (
                  <div className="absolute top-0 left-0 flex h-full w-full items-center justify-center">
                    <div className="flex h-full  flex-col items-center justify-center">
                      <Spinner />
                    </div>
                  </div>
                )}

                <canvas ref={canvasRef} className={`h-full w-full transition ${loadedClass}`} />
              </div>
            </div>

            <div className="h-full">
              <InspectMenu />
            </div>
          </Split>
        </div>
      </DndProvider>
    </>
  );
}
