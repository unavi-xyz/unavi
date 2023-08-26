import { editorStore } from "@unavi/engine";
import { useAtom } from "jotai";

import { usePlayStore } from "@/app/play/playStore";
import { LeftPanelPage } from "@/app/play/types";

import { useTreeValue } from "../../hooks/useTreeValue";
import { getDisplayName } from "../../utils/getDisplayName";
import PanelPage from "../PanelPage";
import SceneTree from "./SceneTree";

export default function ScenePage() {
  const [rootId] = useAtom(editorStore.rootId);
  const [sceneTreeId, setSceneTreeId] = useAtom(editorStore.sceneTreeId);

  const usedId = sceneTreeId || rootId;

  const parentId = useTreeValue(usedId, "parentId");
  const name = useTreeValue(usedId, "name");

  if (usedId === undefined) {
    return null;
  }

  const handleBack = parentId ? () => setSceneTreeId(parentId) : undefined;

  const displayName =
    usedId === rootId ? "Scene" : getDisplayName(name, usedId);

  return (
    <PanelPage title={displayName} onBack={handleBack}>
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
