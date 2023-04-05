"use client";

import { useResizeCanvas } from "@wired-labs/react-client";
import { Engine } from "engine";
import { Metadata } from "next";
import Script from "next/script";
import { useEffect, useRef, useState } from "react";
import Split from "react-split";

import EditorNavbar from "@/src/editor/components/EditorNavbar/EditorNavbar";
import InspectMenu from "@/src/editor/components/InspectMenu/InspectMenu";
import ScriptMenu from "@/src/editor/components/ScriptMenu/ScriptMenu";
import TreeMenu from "@/src/editor/components/TreeMenu/TreeMenu";
import { useAutosave } from "@/src/editor/hooks/useAutosave";
import { ERROR_NOT_SIGNED_IN, useLoad } from "@/src/editor/hooks/useLoad";
import { useTransformControls } from "@/src/editor/hooks/useTransformControls";
import { ERROR_MESSAGE } from "@/src/editor/utils/parseError";
import Crosshair from "@/src/play/Crosshair";
import { Project } from "@/src/server/helpers/fetchProject";

import SignInButton from "../../(navbar)/SignInButton";
import { useEditorStore } from "./store";

export const metadata: Metadata = {
  title: "Editor",
};

interface Props {
  project: Project;
}

export default function Editor({ project }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);

  const engine = useEditorStore((state) => state.engine);
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);
  const openScriptId = useEditorStore((state) => state.openScriptId);
  const isPlaying = useEditorStore((state) => state.isPlaying);
  const [scriptsReady, setScriptsReady] = useState(false);

  const resize = useResizeCanvas(engine, canvasRef, overlayRef, containerRef);
  const { error } = useLoad(project);
  useAutosave(project.id);
  useTransformControls();

  useEffect(() => {
    if (!scriptsReady || !canvasRef.current || !overlayRef.current) return;

    const { showColliders } = useEditorStore.getState();

    const engine = new Engine({
      canvas: canvasRef.current,
      overlayCanvas: overlayRef.current,
    });

    engine.controls = "orbit";
    engine.showColliders = showColliders;

    engine.render.send({ subject: "set_animations_path", data: "/models" });
    engine.render.send({ subject: "set_default_avatar", data: "/models/Robot.vrm" });
    engine.render.send({ subject: "set_skybox", data: { uri: "/images/Skybox.jpg" } });
    engine.start();

    useEditorStore.setState({ engine, canvas: canvasRef.current });

    return () => {
      engine.destroy();
      useEditorStore.setState({
        engine: null,
        canvas: null,
        selectedId: null,
        draggingId: null,
        openScriptId: null,
        isPlaying: false,
        isSaving: false,
        name: "",
        description: "",
        image: null,
        treeIds: [],
        openIds: [],
      });
    };
  }, [scriptsReady]);

  useEffect(() => {
    resize();
  }, [resize, openScriptId]);

  return (
    <>
      <Script src="/scripts/draco_decoder.js" onReady={() => setScriptsReady(true)} />
      <Script src="/scripts/draco_encoder.js" />

      <div
        className="h-full w-full"
        onDragOver={(e) => e.preventDefault()}
        onDrop={async (e) => {
          if (!engine) return;

          e.preventDefault();

          const files = Array.from(e.dataTransfer.files);

          const glbs = files.filter((file) => file.name.endsWith(".glb"));
          const other = files.filter((file) => !file.name.endsWith(".glb"));

          glbs.forEach((file) => engine.scene.addFile(file));
          if (other.length) engine.scene.addFiles(other);
        }}
      >
        <div className="h-12 w-full border-b">
          <EditorNavbar project={project} />
        </div>

        <div className="fixed h-full w-full animate-fadeInDelayed">
          <Split
            sizes={openScriptId ? [50, 50] : [100, 0]}
            minSize={openScriptId ? [250, 250] : [250, 0]}
            direction="vertical"
            gutterSize={8}
            className="h-full w-full"
            onMouseUp={resize}
          >
            <Split
              sizes={[15, 65, 25]}
              minSize={[50, 400, 50]}
              direction="horizontal"
              gutterSize={4}
              className={`flex w-full transition ${openScriptId ? "h-1/2" : "h-full"}`}
              onMouseUp={resize}
            >
              <div>
                <TreeMenu />
              </div>

              {error ? (
                <div className="space-y-2 border-x bg-neutral-100 pt-10 text-center">
                  {error === ERROR_NOT_SIGNED_IN ? (
                    <h2>{error}</h2>
                  ) : error === ERROR_MESSAGE.UNAUTHORIZED ? (
                    <h2>Project not found.</h2>
                  ) : (
                    <h2>Failed to load project. {error}</h2>
                  )}

                  {error === ERROR_NOT_SIGNED_IN ? (
                    <SignInButton />
                  ) : (
                    <button
                      onClick={() => window.location.reload()}
                      className="rounded-lg border border-neutral-500 px-4 py-1 hover:bg-neutral-200 active:bg-neutral-300"
                    >
                      Try again
                    </button>
                  )}
                </div>
              ) : (
                <div className={`border-x bg-neutral-300 ${sceneLoaded ? "" : "animate-pulse"}`}>
                  <div
                    ref={containerRef}
                    className={`relative h-full w-full overflow-hidden transition ${
                      sceneLoaded ? "opacity-100" : "opacity-0"
                    }`}
                  >
                    {isPlaying ? <Crosshair /> : null}

                    <canvas ref={canvasRef} className="h-full w-full" />
                    <canvas ref={overlayRef} className="absolute top-0 left-0 z-10 h-full w-full" />
                  </div>
                </div>
              )}

              <div className="overflow-y-auto">
                <InspectMenu projectId={project.id} />
              </div>
            </Split>

            <div>{openScriptId && <ScriptMenu key={openScriptId} scriptId={openScriptId} />}</div>
          </Split>
        </div>
      </div>
    </>
  );
}
