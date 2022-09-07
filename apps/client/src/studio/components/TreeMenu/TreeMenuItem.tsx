import { useRef, useState } from "react";
import { useDrag, useDrop } from "react-dnd";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

import { useStudioStore } from "../../../studio/store";
import { DND_TYPES } from "../../../studio/types";
import { moveEntity } from "../../actions/MoveEntityAction";
import { moveToSibling } from "../../utils/tree";

type DragItem = {
  id: string;
};

interface Props {
  id: string;
}

export default function TreeMenuItem({ id }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const selectedId = useStudioStore((state) => state.selectedId);
  const name = useStudioStore((state) => state.tree[id].name);
  const children = useStudioStore((state) => state.tree[id].children);
  const [open, setOpen] = useState(true);

  // Create drag source
  const [{ isDragging }, drag] = useDrag(
    () => ({
      type: DND_TYPES.TreeItem,
      item: { id },
      collect: (monitor) => ({
        isDragging: monitor.isDragging(),
      }),
    }),
    [id]
  );

  // Create drop target
  const [{ isOver }, drop] = useDrop(
    () => ({
      accept: DND_TYPES.TreeItem,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          if (isDeepChild(id, droppedId)) return;

          // Move to new parent
          moveEntity(droppedId, id);
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver({ shallow: true }),
      }),
    }),
    [id]
  );

  // Create drop target for below
  const [{ isOver: isOverBelow }, dropBelow] = useDrop(
    () => ({
      accept: DND_TYPES.TreeItem,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          if (isDeepChild(id, droppedId)) return;

          // Add as sibling
          moveToSibling(droppedId, id, "below");
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver({ shallow: true }),
      }),
    }),
    [id]
  );

  const hasChildren = children.length > 0;
  const isSelected = selectedId === id;
  const bgClass = isSelected
    ? "bg-primaryContainer text-onPrimaryContainer"
    : isOver
    ? "bg-surfaceVariant"
    : "hover:bg-surfaceVariant";
  const opacityClass = isDragging ? "opacity-0" : null;
  const highlightBelowClass = isOverBelow ? "bg-primary" : null;

  drop(ref);
  drag(ref);

  return (
    <div ref={ref} className="h-full">
      <div
        onMouseDown={(e) => {
          e.stopPropagation();
          useStudioStore.setState({ selectedId: id });
        }}
        className={`h-6 font-bold rounded-md px-2 flex items-center ${bgClass} ${opacityClass}`}
      >
        <div
          onClick={() => setOpen((prev) => !prev)}
          className="w-5 text-outline hover:text-inherit transition"
        >
          {hasChildren &&
            (open ? <IoMdArrowDropdown /> : <IoMdArrowDropright />)}
        </div>

        <div>{name}</div>
      </div>

      {open && hasChildren && (
        <div className={`pl-4 ${opacityClass}`}>
          {children.map((child) => (
            <TreeMenuItem key={child} id={child} />
          ))}
        </div>
      )}

      {!hasChildren && (
        <div
          ref={dropBelow}
          className={`-mt-1 w-full h-1 rounded ${highlightBelowClass}`}
        />
      )}
    </div>
  );
}

function isDeepChild(entityId: string, parentId: string) {
  const { tree } = useStudioStore.getState();
  const object = tree[parentId];

  if (object.children.includes(entityId)) return true;

  for (const child of object.children) {
    if (isDeepChild(entityId, child)) return true;
  }

  return false;
}
