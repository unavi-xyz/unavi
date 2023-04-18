"use client";

import { useState } from "react";
import { Panel, PanelGroup, PanelResizeHandle } from "react-resizable-panels";

import { Project } from "@/src/server/helpers/fetchProject";

import { useEditor } from "./Editor";
import EditorCanvas from "./EditorCanvas";
import EditorNavbar from "./EditorNavbar/EditorNavbar";
import InspectMenu from "./InspectMenu/InspectMenu";
import ScriptMenu from "./ScriptMenu/ScriptMenu";
import TreeMenu from "./TreeMenu/TreeMenu";

interface Props {
  project: Project;
}

export default function EditorUI({ project }: Props) {
  const [resize, setResize] = useState<() => void>();

  const { engine, scriptId } = useEditor();

  return (
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
        <PanelGroup autoSaveId="editor-script" direction="vertical">
          <Panel onResize={resize} collapsible minSize={20}>
            <PanelGroup autoSaveId="editor-inspect" direction="horizontal">
              <Panel onResize={resize}>
                <PanelGroup autoSaveId="editor-tree" direction="horizontal">
                  <Panel collapsible defaultSize={15} minSize={10}>
                    <TreeMenu />
                  </Panel>

                  <PanelResizeHandle className="group w-2 border-r p-[1px]">
                    <div className="h-full rounded-full transition duration-300 group-active:bg-neutral-300" />
                  </PanelResizeHandle>

                  <Panel onResize={resize} minSize={30}>
                    <EditorCanvas setResize={setResize} />
                  </Panel>
                </PanelGroup>
              </Panel>

              <PanelResizeHandle className="group w-2 border-l p-[1px]">
                <div className="h-full rounded-full transition duration-300 group-active:bg-neutral-300" />
              </PanelResizeHandle>

              <Panel collapsible defaultSize={20} minSize={15}>
                <InspectMenu projectId={project.id} />
              </Panel>
            </PanelGroup>
          </Panel>

          {scriptId ? (
            <>
              <PanelResizeHandle className="group h-2 border-t p-[1px]">
                <div className="h-full w-full rounded-full transition duration-300 group-active:bg-neutral-300" />
              </PanelResizeHandle>

              <Panel defaultSize={60}>
                <ScriptMenu />
              </Panel>
            </>
          ) : null}
        </PanelGroup>
      </div>
    </div>
  );
}
