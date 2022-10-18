import { Entity, GLTFMesh, SceneJSON } from "@wired-labs/engine";
import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useMemo, useRef, useState } from "react";
import { MdClose } from "react-icons/md";

import { trpc } from "../../../client/trpc";
import { useEditorStore } from "../../../editor/store";
import { SavedSceneJSON } from "../../../editor/types";
import {
  imageStorageKey,
  modelStorageKey,
} from "../../../editor/utils/fileStorage";
import MetaTags from "../../../home/MetaTags";
import Spinner from "../../../ui/Spinner";

export default function Preview() {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const createdEngine = useRef(false);

  const [exportedScene, setExportedScene] = useState<Uint8Array>();

  const engine = useEditorStore((state) => state.engine);

  const router = useRouter();
  const id = router.query.id as string;

  const { data: sceneURL } = trpc.useQuery(["auth.project-scene", { id }], {
    enabled: id !== undefined,
    cacheTime: 0,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    refetchOnMount: false,
  });

  const { data: fileURLs } = trpc.useQuery(["auth.project-files", { id }], {
    enabled: id !== undefined,
    cacheTime: 0,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    refetchOnMount: false,
  });

  useEffect(() => {
    if (createdEngine.current) return;
    createdEngine.current = true;

    async function initEngine() {
      const canvas = canvasRef.current;
      if (!canvas) throw new Error("Canvas not found");

      const { Engine } = await import("@wired-labs/engine");

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
      const { colliders } = useEditorStore.getState();
      engine.renderThread.postMessage({
        subject: "show_visuals",
        data: {
          visible: colliders,
        },
      });

      useEditorStore.setState({ engine });
    }

    initEngine();
  }, [canvasRef]);

  // Load scene on query fetch
  useEffect(() => {
    async function load() {
      if (!engine || !sceneURL || !fileURLs) return;

      const sceneResponse = await fetch(sceneURL);
      const savedScene: SavedSceneJSON = await sceneResponse.json();

      const scene: SceneJSON = {
        spawn: savedScene.spawn,
        accessors: savedScene.accessors,
        animations: savedScene.animations,
        entities: savedScene.entities,
        materials: savedScene.materials,
        images: [],
      };

      // Load glTF models
      const modelPromises = savedScene.entities.map(async (entity) => {
        if (entity.mesh?.type === "glTF" && entity.mesh.uri) {
          const file = fileURLs.find(
            (f) => f.id === modelStorageKey(entity.id)
          );
          if (!file) throw new Error("File not found");

          const response = await fetch(file.uri);
          const blob = await response.blob();
          const url = URL.createObjectURL(blob);

          entity.mesh.uri = url;
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
      });

      await Promise.all(modelPromises);
      await Promise.all(imagePromises);

      // Load scene
      await engine.scene.loadJSON(scene);

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

    // Load exported scene
    const blob = new Blob([exportedScene], { type: "model/gltf-binary" });
    const url = URL.createObjectURL(blob);

    // Create glTF entity
    const entity = new Entity();
    const mesh = new GLTFMesh();
    mesh.uri = url;
    entity.mesh = mesh;

    // Add entity to scene
    engine.scene
      .loadJSON({
        entities: [entity.toJSON()],
      })
      .then(() => {
        // Start engine
        engine.start();
      });

    return () => {
      engine.scene.removeEntity(entity.id);
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

      <div className="h-full">
        {exportedScene ? (
          <div className="crosshair" />
        ) : (
          <div className="absolute top-0 left-0 flex h-full w-full items-center justify-center">
            <div className="flex h-full flex-col items-center justify-center">
              <Spinner />
            </div>
          </div>
        )}

        <div
          ref={containerRef}
          className="relative h-full w-full overflow-hidden"
        >
          <canvas
            ref={canvasRef}
            className={`h-full w-full transition ${loadedClass}`}
          />
        </div>

        <div
          onClick={(e) => e.stopPropagation()}
          className="fixed top-0 right-0 p-6 text-2xl"
        >
          <Link href={`/editor/${id}`} passHref>
            <button className="rounded-full bg-surface p-2 text-onSurface shadow transition hover:shadow-lg active:shadow">
              <MdClose />
            </button>
          </Link>
        </div>
      </div>
    </>
  );
}
