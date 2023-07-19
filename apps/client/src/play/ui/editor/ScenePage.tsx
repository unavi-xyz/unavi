import { useSceneStore } from "@unavi/react-client";

import { usePlayStore } from "@/app/play/playStore";
import { LeftPanelPage } from "@/app/play/types";

import { useTreeItem } from "./hooks/useTreeItem";
import PanelPage from "./PanelPage";
import SceneTree from "./SceneTree";

export default function ScenePage() {
  const rootId = useSceneStore((state) => state.rootId);
  const sceneTreeId = useSceneStore((state) => state.sceneTreeId);
  const usedId = sceneTreeId || rootId;

  const item = useTreeItem(usedId);

  if (usedId === undefined) {
    return null;
  }

  const handleBack = item?.parentId
    ? () => useSceneStore.setState({ sceneTreeId: item.parentId })
    : undefined;

  const name =
    usedId === rootId ? "Scene" : item?.name || `(${usedId.toString()})`;

  return (
    <PanelPage title={name} onBack={handleBack}>
      <button
        onClick={() => usePlayStore.setState({ leftPage: LeftPanelPage.Add })}
        className="w-full rounded-lg bg-white/10 px-4 py-2 transition hover:bg-white/20 active:opacity-80"
      >
        Add
      </button>

      <SceneTree rootId={usedId} />
    </PanelPage>
  );
}
