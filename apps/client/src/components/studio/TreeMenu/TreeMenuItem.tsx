import { useState } from "react";
import { useDrag, useDrop } from "react-dnd";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

import { findObjectById, TreeObject } from "scene";

import { useStudioStore } from "../../../helpers/studio/store";
import { DND_TYPES } from "../../../helpers/studio/types";

type DragItem = {
  id: string;
};

interface Props {
  object: TreeObject;
  isRoot?: boolean;
}

export default function TreeMenuItem({ object, isRoot = false }: Props) {
  const tree = useStudioStore((state) => state.scene.tree);
  const selectedId = useStudioStore((state) => state.selectedId);
  const moveObject = useStudioStore((state) => state.moveObject);

  const [open, setOpen] = useState(true);

  const [, drag] = useDrag<DragItem>(
    () => ({
      type: DND_TYPES.TreeObject,
      item: { id: object.id },
    }),
    [object]
  );

  const [, drop] = useDrop(
    () => ({
      accept: DND_TYPES.TreeObject,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== object.id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          const dropped = findObjectById(tree, droppedId);
          if (!dropped) return;

          moveObject(droppedId, object.id);
        }
      },
    }),
    [object, tree, moveObject]
  );

  const hasChildren = object.children.length > 0;
  const isSelected = selectedId === object.id;
  const selectedClass = isSelected ? "bg-neutral-100" : "";
  const paddingClass = isRoot ? "" : "pl-4";

  return (
    <div
      ref={drop}
      onClick={(e) => {
        e.stopPropagation();
        if (isRoot) useStudioStore.setState({ selectedId: undefined });
      }}
      className={`space-y-1 h-full ${paddingClass}`}
    >
      {!isRoot && (
        <div
          ref={drag}
          onClick={() => useStudioStore.setState({ selectedId: object.id })}
          className={`font-bold hover:bg-neutral-100 rounded-md px-2 cursor-pointer
                        flex items-center h-8 ${selectedClass}`}
        >
          <div
            onClick={() => setOpen((prev) => !prev)}
            className="w-5 text-neutral-500 hover:text-neutral-400"
          >
            {hasChildren &&
              (open ? <IoMdArrowDropdown /> : <IoMdArrowDropright />)}
          </div>

          <div>{object.name}</div>
        </div>
      )}

      <div className="space-y-1">
        {open &&
          object.children.map((child) => {
            return <TreeMenuItem key={child.id} object={child} />;
          })}
      </div>
    </div>
  );
}
