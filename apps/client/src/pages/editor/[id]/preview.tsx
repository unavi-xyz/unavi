import { GLTFMesh, Node, SceneJSON } from "engine";
import Link from "next/link";
import { useRouter } from "next/router";
import Script from "next/script";
import { useEffect, useMemo, useRef, useState } from "react";
import { MdClose } from "react-icons/md";

import LoadingScreen from "../../../app/ui/LoadingScreen";
import { trpc } from "../../../client/trpc";
import { useEditorStore } from "../../../editor/store";
import { SavedSceneJSON } from "../../../editor/types";
import { binaryStorageKey, imageStorageKey } from "../../../editor/utils/fileStorage";
import { updateGltfColliders } from "../../../editor/utils/updateGltfColliders";
import MetaTags from "../../../home/MetaTags";

export default function Preview() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const createdEngine = useRef(false);

  const [exportedScene, setExportedScene] = useState<Uint8Array>();
  const [loadingProgress, setLoadingProgress] = useState(0);
  const [loadingText, setLoadingText] = useState("Starting engine...");

  const engine = useEditorStore((state) => state.engine);

  const router = useRouter();
  const id = router.query.id as string;

  const { data: project } = trpc.project.get.useQuery(
    { id },
    {
      enabled: id !== undefined,
      cacheTime: 0,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      refetchOnMount: false,
      trpc: {},
    }
  );

  const { data: imageURL } = trpc.project.image.useQuery(
    { id },
    {
      enabled: id !== undefined && !project?.publicationId,
      trpc: {},
    }
  );

  const { data: sceneURL } = trpc.project.scene.useQuery(
    { id },
    {
      enabled: id !== undefined,
      cacheTime: 0,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      refetchOnMount: false,
      trpc: {},
    }
  );

  const { data: fileURLs } = trpc.project.files.useQuery(
    { id },
    {
      enabled: id !== undefined,
      cacheTime: 0,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      refetchOnMount: false,
      trpc: {},
    }
  );

  useEffect(() => {
    if (!engine || !project?.editorState) return;

    const editorState = JSON.parse(project.editorState);
    if (!editorState) return;

    engine.waitForReady().then(() => {
      engine.renderThread.postMessage({
        subject: "show_visuals",
        data: { visible: editorState.visuals },
      });
    });
  }, [engine, project]);

  useEffect(() => {
    if (createdEngine.current) return;
    createdEngine.current = true;

    async function initEngine() {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error("Canvas not found");

      setLoadingText("Starting engine...");
      setLoadingProgress(0);

      const { Engine } = await import("engine");

      // Create engine
      const engine = new Engine({
        canvas,
        camera: "player",
        skyboxPath: "/images/skybox/",
        avatarPath: "/models/Wired-chan.vrm",
        avatarAnimationsPath: "/models/",
      });

      await engine.waitForReady();

      engine.setAvatar(null);

      // Set collider visibility
      const { visuals } = useEditorStore.getState();
      engine.renderThread.postMessage({
        subject: "show_visuals",
        data: { visible: visuals },
      });

      useEditorStore.setState({ engine });
    }

    initEngine();
  }, [canvasRef]);

  // Load scene on query fetch
  useEffect(() => {
    async function load() {
      if (!engine || !sceneURL || !fileURLs) return;

      setLoadingText("Fetching scene...");
      setLoadingProgress(0.2);

      const sceneResponse = await fetch(sceneURL);
      const savedScene: SavedSceneJSON = await sceneResponse.json();

      setLoadingProgress(0.25);

      const scene: SceneJSON = {
        ...savedScene,
        images: [],
        accessors: [],
      };

      const fetchCount = Object.keys(fileURLs).length;
      const fetchPercent = 0.15 / fetchCount;

      // Load glTF models
      const modelPromises = scene.meshes.map(async (mesh) => {
        if (mesh?.type === "glTF" && mesh.uri) {
          const file = fileURLs.find((f) => f.id === binaryStorageKey(mesh.id));
          if (!file) throw new Error("File not found");

          const response = await fetch(file.uri);
          const blob = await response.blob();
          const url = URL.createObjectURL(blob);

          mesh.uri = url;

          setLoadingProgress((prev) => prev + fetchPercent);
        }
      });

      // Load images
      const imagePromises = savedScene.images.map(async (image) => {
        const file = fileURLs.find((f) => f.id === imageStorageKey(image.id));
        if (!file) throw new Error("File not found");

        const response = await fetch(file.uri);
        const buffer = await response.arrayBuffer();
        const array = new Uint8Array(buffer);

        const blob = new Blob([array], { type: image.mimeType });
        const bitmap = await createImageBitmap(blob);

        scene.images.push({
          id: image.id,
          isInternal: false,
          mimeType: image.mimeType,
          array,
          bitmap,
        });

        setLoadingProgress((prev) => prev + fetchPercent);
      });

      // Load accessors
      savedScene.accessors.map((accessor) => {
        const array =
          accessor.type === "SCALAR"
            ? new Uint16Array(accessor.array)
            : new Float32Array(accessor.array);

        scene.accessors.push({
          ...accessor,
          array,
          isInternal: false,
        });
      });

      await Promise.all(modelPromises);
      await Promise.all(imagePromises);

      setLoadingText("Loading scene...");

      // Load scene
      await engine.scene.loadJSON(scene);

      // Update colliders
      scene.nodes.forEach((node) => updateGltfColliders(node.id));

      setLoadingText("Exporting scene...");
      setLoadingProgress(0.6);

      // Export scene as GLB
      const glb = await engine.export();

      // Clear scene
      engine.scene.clear();

      // Set exported scene
      setExportedScene(glb);
    }

    load();
  }, [engine, sceneURL, fileURLs]);

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

  useEffect(() => {
    if (!engine || !exportedScene) return;

    setLoadingText("Loading exported scene...");
    setLoadingProgress(0.8);

    // Load exported scene
    const blob = new Blob([exportedScene], { type: "model/gltf-binary" });
    const url = URL.createObjectURL(blob);

    // Create glTF node
    const mesh = new GLTFMesh();
    mesh.uri = url;

    const node = new Node();
    node.meshId = mesh.id;

    // Add node to scene
    engine.scene
      .loadJSON(
        {
          nodes: [node.toJSON()],
          meshes: [mesh.toJSON()],
        },
        true
      )
      .then(async () => {
        // Start engine
        await engine.start();

        setLoadingText("Ready!");
        setLoadingProgress(1);
      });

    return () => {
      engine.scene.removeMesh(mesh.id);
      engine.scene.removeNode(node.id);
    };
  }, [engine, exportedScene]);

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

  const loadedClass = exportedScene ? "opacity-100" : "opacity-0";

  return (
    <>
      <MetaTags title="Preview" />

      <Script src="/scripts/draco_encoder.js" />
      <Script src="/scripts/draco_decoder.js" />

      <div className="h-screen w-screen overflow-hidden">
        <LoadingScreen
          text={project?.name}
          image={imageURL}
          loaded={loadingProgress === 1}
          loadingText={loadingText}
          loadingProgress={loadingProgress}
        />

        <div ref={containerRef} className="relative h-full w-full overflow-hidden">
          <canvas ref={canvasRef} className={`h-full w-full transition ${loadedClass}`} />
        </div>

        <div onClick={(e) => e.stopPropagation()} className="fixed top-0 right-0 p-6 text-2xl">
          <Link href={`/editor/${id}`}>
            <button className="rounded-full bg-white p-2 shadow transition hover:shadow-md active:shadow">
              <MdClose />
            </button>
          </Link>
        </div>
      </div>
    </>
  );
}
