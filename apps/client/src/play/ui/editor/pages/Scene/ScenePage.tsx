import { syncedStore, useSceneStore } from "@unavi/engine";
import { useSnapshot } from "valtio";

import { usePlayStore } from "@/app/play/playStore";
import { LeftPanelPage } from "@/app/play/types";

import { getDisplayName } from "../../utils/getDisplayName";
import PanelPage from "../PanelPage";
import SceneTree from "./SceneTree";

export default function ScenePage() {
  const snap = useSnapshot(syncedStore);

  const rootId = useSceneStore((state) => state.rootId);
  const sceneTreeId = useSceneStore((state) => state.sceneTreeId);
  const id = sceneTreeId || rootId;

  if (!id) {
    return null;
  }

  const node = snap.nodes[id];
  const scene = snap.scenes[id];

  const isRoot = id === rootId;
  const obj = isRoot ? scene : node;

  if (!obj) {
    return null;
  }

  let handleBack = undefined;

  if (node) {
    const parentId = node.parentId || rootId;
    handleBack = () => useSceneStore.setState({ sceneTreeId: parentId });
  }

  const displayName = id === rootId ? "Scene" : getDisplayName(obj.name, id);

  return (
    <PanelPage title={displayName} onBack={handleBack}>
      <button
        onClick={() => usePlayStore.setState({ leftPage: LeftPanelPage.Add })}
        className="w-full rounded-lg bg-white/10 px-4 py-2 transition hover:bg-white/20 active:opacity-80"
      >
        Add
      </button>

      <SceneTree obj={obj} />
    </PanelPage>
  );
}
