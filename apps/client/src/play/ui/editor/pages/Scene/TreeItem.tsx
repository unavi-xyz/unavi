import { SyncedNode, syncedStore, useSceneStore } from "@unavi/engine";
import { IoMdExpand, IoMdLock, IoMdUnlock } from "react-icons/io";

import { DeepReadonly } from "@/src/play/utils/types";
import Tooltip from "@/src/ui/Tooltip";

import { getDisplayName } from "../../utils/getDisplayName";

interface Props {
  node: DeepReadonly<SyncedNode>;
}

export default function TreeItem({ node }: Props) {
  const selectedId = useSceneStore((state) => state.selectedId);

  function select(e: React.MouseEvent) {
    e.stopPropagation();
    useSceneStore.setState({ selectedId: node.id });
  }

  function expand(e: React.MouseEvent) {
    e.stopPropagation();
    useSceneStore.setState({ sceneTreeId: node.id });
  }

  function toggleLock(e: React.MouseEvent) {
    e.stopPropagation();

    const synced = syncedStore.nodes[node.id];
    if (!synced) return;

    synced.extras.locked = !synced.extras.locked;
  }

  const isSelected = selectedId === node.id;

  return (
    <div className="group relative flex space-x-1">
      <button
        onClick={select}
        className={`w-full rounded px-2 py-0.5 text-start active:opacity-90 ${
          isSelected
            ? "bg-white/10 group-hover:bg-white/20"
            : "group-hover:bg-white/10"
        }`}
      >
        {getDisplayName(node.name, node.id)}
      </button>

      <div className="absolute inset-y-0 right-0 flex items-center space-x-1 pr-1">
        <Tooltip text="Expand" side="top">
          <button
            onClick={expand}
            className="hidden rounded text-lg hover:opacity-70 active:opacity-60 group-hover:block"
          >
            <IoMdExpand />
          </button>
        </Tooltip>

        <Tooltip text={node.extras.locked ? "Unlock" : "Lock"} side="top">
          <button
            onClick={toggleLock}
            className={`rounded text-lg hover:opacity-70 active:opacity-60 ${
              node.extras.locked
                ? "text-neutral-500"
                : "hidden group-hover:block"
            }`}
          >
            {node.extras.locked ? <IoMdLock /> : <IoMdUnlock />}
          </button>
        </Tooltip>
      </div>
    </div>
  );
}
