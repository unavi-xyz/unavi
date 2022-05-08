import { useEffect } from "react";
import { useRouter } from "next/router";
import Head from "next/head";
import { DndProvider } from "react-dnd";
import { HTML5Backend } from "react-dnd-html5-backend";

import { useAutosave } from "../../../src/helpers/studio/hooks/useAutosave";
import { useHotkeys } from "../../../src/helpers/studio/hooks/useStudioHotkeys";
import { useStudioStore } from "../../../src/helpers/studio/store";

import StudioNavbar from "../../../src/components/studio/StudioNavbar/StudioNavbar";
import StudioCanvas from "../../../src/components/studio/StudioCanvas/StudioCanvas";
import InspectMenu from "../../../src/components/studio/InspectMenu/InspectMenu";
import TreeMenu from "../../../src/components/studio/TreeMenu/TreeMenu";

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
    <DndProvider backend={HTML5Backend}>
      <div className="h-full overflow-hidden">
        <Head>
          <title>Studio · The Wired</title>
        </Head>

        <div className="bg-white border-b w-full h-14 z-10">
          <StudioNavbar />
        </div>

        <div className="flex h-full">
          <div className="fixed left-0 bg-white border-r h-full w-[250px] z-10">
            <TreeMenu />
          </div>

          <div className="bg-neutral-100 w-full pl-[250px] pr-[350px]">
            <StudioCanvas />
          </div>

          <div className="fixed right-0 bg-white border-l h-full w-[350px] z-10">
            <InspectMenu />
          </div>
        </div>
      </div>
    </DndProvider>
  );
}
