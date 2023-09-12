import { useSceneStore } from "@unavi/engine";
import { IoMdExpand, IoMdLock, IoMdUnlock } from "react-icons/io";

import { editNode } from "@/src/play/actions/editNode";
import Tooltip from "@/src/ui/Tooltip";

import { useTreeValue } from "../../hooks/useTreeValue";

interface Props {
  entityId: bigint;
}

export default function TreeItem({ entityId }: Props) {
  const selectedId = useSceneStore((state) => state.selectedId);
  const id = useTreeValue(entityId, "id");
  const name = useTreeValue(entityId, "name");
  const locked = useTreeValue(entityId, "locked");

  function select(e: React.MouseEvent) {
    e.stopPropagation();
    useSceneStore.setState({ selectedId: entityId });
  }

  function expand(e: React.MouseEvent) {
    e.stopPropagation();
    useSceneStore.setState({ sceneTreeId: entityId });
  }

  function toggleLock(e: React.MouseEvent) {
    e.stopPropagation();
    if (!id) return;
    editNode(id, { extras: { locked: !locked } });
  }

  const isSelected = selectedId === entityId;

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
        {name || `(${entityId.toString()})`}
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

        <Tooltip text={locked ? "Unlock" : "Lock"} side="top">
          <button
            onClick={toggleLock}
            className={`rounded text-lg hover:opacity-70 active:opacity-60 ${
              locked ? "text-neutral-500" : "hidden group-hover:block"
            }`}
          >
            {locked ? <IoMdLock /> : <IoMdUnlock />}
          </button>
        </Tooltip>
      </div>
    </div>
  );
}
