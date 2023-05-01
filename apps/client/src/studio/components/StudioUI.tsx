import { Project } from "@/src/server/helpers/fetchProject";

import GlobalFileDrop from "./GlobalFileDrop";
import InspectMenu from "./InspectMenu/InspectMenu";
import { Panel, PanelGroup, PanelResizeHandle } from "./Panel";
import ScriptPanel from "./ScriptPanel";
import StudioCanvas from "./StudioCanvas";
import StudioNavbar from "./StudioNavbar/StudioNavbar";
import TreeMenu from "./TreeMenu/TreeMenu";

interface Props {
  project: Project;
}

export default function StudioUI({ project }: Props) {
  return (
    <GlobalFileDrop>
      <StudioNavbar project={project} />

      <div className="fixed h-full w-full animate-fadeInDelayed">
        <PanelGroup autoSaveId="studio-script" direction="vertical">
          <Panel collapsible minSize={20}>
            <PanelGroup autoSaveId="studio-inspect" direction="horizontal">
              <Panel>
                <PanelGroup autoSaveId="studio-tree" direction="horizontal">
                  <Panel collapsible defaultSize={15} minSize={10}>
                    <TreeMenu />
                  </Panel>

                  <PanelResizeHandle className="group w-2 border-r p-[1px]">
                    <div className="h-full rounded-full transition duration-300 group-active:bg-neutral-300" />
                  </PanelResizeHandle>

                  <Panel minSize={30}>
                    <StudioCanvas />
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

          <ScriptPanel />
        </PanelGroup>
      </div>
    </GlobalFileDrop>
  );
}
