import {
  SyncedNode,
  SyncedScene,
  syncedStore,
  useSceneStore,
} from "@unavi/engine";
import { useSnapshot } from "valtio";

import { DeepReadonly } from "@/src/play/utils/types";

import { getChildren } from "../../utils/getChildren";
import TreeItem from "./TreeItem";

interface Props {
  obj: DeepReadonly<SyncedNode | SyncedScene>;
}

export default function SceneTree({ obj }: Props) {
  const snap = useSnapshot(syncedStore);
  const children = getChildren(obj.id, snap);

  function clearSelected() {
    useSceneStore.setState({ selectedId: undefined });
  }

  return (
    <div onClick={clearSelected} className="h-full space-y-1">
      {children.map((node) => (
        <TreeItem key={node.id} node={node} />
      ))}
    </div>
  );
}
