import { useEffect, useState } from "react";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";
import Split from "react-split";

import FileBrowser from "../../src/studio/components/FileBrowser/FileBrowser";
import InspectMenu from "../../src/studio/components/InspectMenu/InspectMenu";
import ProjectsPage from "../../src/studio/components/ProjectsPage/ProjectsPage";
import StudioCanvas from "../../src/studio/components/StudioCanvas/StudioCanvas";
import StudioNavbar from "../../src/studio/components/StudioNavbar/StudioNavbar";
import TreeMenu from "../../src/studio/components/TreeMenu/TreeMenu";
import { useAutosave } from "../../src/studio/hooks/useAutosave";
import { useHotkeys } from "../../src/studio/hooks/useStudioHotkeys";
import { useStudioStore } from "../../src/studio/store";
import MetaTags from "../../src/ui/MetaTags";
import Dialog from "../../src/ui/base/Dialog";

export default function Studio() {
  const rootHandle = useStudioStore((state) => state.rootHandle);

  const [open, setOpen] = useState(false);

  useEffect(() => {
    setOpen(!Boolean(rootHandle));
  }, [rootHandle]);

  useAutosave();
  useHotkeys();

  return (
    <>
      <MetaTags title="Studio" />

      <Dialog open={open}>
        <ProjectsPage />
      </Dialog>

      <DndProvider backend={HTML5Backend}>
        <div className="h-full overflow-hidden flex flex-col">
          <div className="border-b w-full h-14 z-10">
            <StudioNavbar />
          </div>

          <Split
            sizes={[70, 30]}
            minSize={200}
            direction="vertical"
            expandToMin
            gutterSize={6}
            className="h-full"
          >
            <div className="h-full">
              <Split
                sizes={[15, 65, 20]}
                minSize={200}
                direction="horizontal"
                expandToMin
                gutterSize={6}
                className="h-full"
              >
                <div className="float-left h-full">
                  <TreeMenu />
                </div>

                <div className="float-left h-full border-x">
                  <StudioCanvas />
                </div>

                <div className="float-left h-full">
                  <InspectMenu />
                </div>
              </Split>
            </div>

            <div>
              <FileBrowser />
            </div>
          </Split>
        </div>
      </DndProvider>
    </>
  );
}
