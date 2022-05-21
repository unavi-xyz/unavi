import Head from "next/head";
import { useRouter } from "next/router";
import { useEffect } from "react";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";

import InspectMenu from "../../../src/components/studio/InspectMenu/InspectMenu";
import StudioCanvas from "../../../src/components/studio/StudioCanvas/StudioCanvas";
import StudioNavbar from "../../../src/components/studio/StudioNavbar/StudioNavbar";
import TreeMenu from "../../../src/components/studio/TreeMenu/TreeMenu";
import { useAutosave } from "../../../src/helpers/studio/hooks/useAutosave";
import { useHotkeys } from "../../../src/helpers/studio/hooks/useStudioHotkeys";
import { useStudioStore } from "../../../src/helpers/studio/store";

export default function Studio() {
  const router = useRouter();
  const id = router.query.id as string;

  useAutosave();
  useHotkeys();

  useEffect(() => {
    useStudioStore.setState({ id });

    return () => {
      useStudioStore.setState({ id: undefined });
    };
  }, [id]);

  return (
    <>
      <Head>
        <title>Studio / The Wired</title>
      </Head>

      <DndProvider backend={HTML5Backend}>
        <div className="h-full overflow-hidden">
          <div className="border-b w-full h-14 z-10">
            <StudioNavbar />
          </div>

          <div className="flex h-full">
            <div className="fixed left-0 border-r h-full w-[250px] z-10">
              <TreeMenu />
            </div>

            <div className="w-full pl-[250px] pr-[400px]">
              <StudioCanvas />
            </div>

            <div className="fixed right-0 border-l h-full w-[400px] z-10">
              <InspectMenu />
            </div>
          </div>
        </div>
      </DndProvider>
    </>
  );
}
