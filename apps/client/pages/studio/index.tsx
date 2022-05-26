import Head from "next/head";
import { useEffect, useState } from "react";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";
import Split from "react-split";

import Dialog from "../../src/components/base/Dialog";
import FileBrowser from "../../src/components/studio/FileBrowser/FileBrowser";
import InspectMenu from "../../src/components/studio/InspectMenu/InspectMenu";
import ProjectsPage from "../../src/components/studio/ProjectsPage/ProjectsPage";
import StudioCanvas from "../../src/components/studio/StudioCanvas/StudioCanvas";
import StudioNavbar from "../../src/components/studio/StudioNavbar/StudioNavbar";
import TreeMenu from "../../src/components/studio/TreeMenu/TreeMenu";
import { useAutosave } from "../../src/helpers/studio/hooks/useAutosave";
import { useHotkeys } from "../../src/helpers/studio/hooks/useStudioHotkeys";
import { useStudioStore } from "../../src/helpers/studio/store";

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
      <Head>
        <title>Studio / The Wired</title>
      </Head>

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
                sizes={[15, 70, 15]}
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
