"use client";

import { useResizeCanvas } from "@wired-labs/react-client";
import { Engine } from "engine";
import { Metadata } from "next";
import Script from "next/script";
import { useEffect, useRef, useState } from "react";
import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";

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
        title: "",
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
          <PanelGroup autoSaveId="editor-vertical" direction="vertical">
            <Panel>
              <PanelGroup autoSaveId="editor-horizontal" direction="horizontal">
                <Panel collapsible defaultSize={15} minSize={10}>
                  <TreeMenu />
                </Panel>

                <PanelResizeHandle className="group w-2 border-r p-[1px]">
                  <div className="h-full rounded-full transition duration-300 group-active:bg-neutral-300" />
                </PanelResizeHandle>

                <Panel minSize={30}>
                  {error ? (
                    <div className="h-full space-y-2 bg-neutral-100 pt-10 text-center">
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
                    <div className={`h-full bg-neutral-300 ${sceneLoaded ? "" : "animate-pulse"}`}>
                      <div
                        ref={containerRef}
                        className={`relative h-full w-full overflow-hidden transition ${
                          sceneLoaded ? "opacity-100" : "opacity-0"
                        }`}
                      >
                        {isPlaying ? <Crosshair /> : null}

                        <canvas ref={canvasRef} className="h-full w-full" />
                        <canvas
                          ref={overlayRef}
                          className="absolute top-0 left-0 z-10 h-full w-full"
                        />
                      </div>
                    </div>
                  )}
                </Panel>

                <PanelResizeHandle className="group w-2 border-l p-[1px]">
                  <div className="h-full rounded-full transition duration-300 group-active:bg-neutral-300" />
                </PanelResizeHandle>

                <Panel collapsible defaultSize={20} minSize={15}>
                  <InspectMenu projectId={project.id} />
                </Panel>
              </PanelGroup>
            </Panel>

            {openScriptId ? (
              <>
                <PanelResizeHandle className="group h-2 border-t p-[1px]">
                  <div className="h-full w-full rounded-full transition duration-300 group-active:bg-neutral-300" />
                </PanelResizeHandle>

                <Panel defaultSize={60}>
                  <ScriptMenu key={openScriptId} scriptId={openScriptId} />
                </Panel>
              </>
            ) : null}
          </PanelGroup>
        </div>
      </div>
    </>
  );
}
