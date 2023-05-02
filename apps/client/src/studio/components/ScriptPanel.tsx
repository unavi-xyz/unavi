"use client";

import { Panel, PanelResizeHandle } from "./Panel";
import ScriptMenu from "./ScriptMenu/ScriptMenu";
import { useStudio } from "./Studio";

export default function ScriptPanel() {
  const { scriptId } = useStudio();

  if (!scriptId) return null;
  return (
    <>
      <PanelResizeHandle className="group h-2 border-t p-[1px]">
        <div className="h-full w-full rounded-full transition duration-300 group-active:bg-neutral-300" />
      </PanelResizeHandle>

      <Panel defaultSize={60}>
        <ScriptMenu />
      </Panel>
    </>
  );
}
