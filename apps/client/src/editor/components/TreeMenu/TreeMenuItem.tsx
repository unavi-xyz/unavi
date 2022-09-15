import { useRef, useState } from "react";
import { useDrag, useDrop } from "react-dnd";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

import { moveEntity } from "../../actions/MoveEntityAction";
import { useEditorStore } from "../../store";
import { DND_TYPES } from "../../types";
import { moveToSibling } from "../../utils/tree";

type DragItem = {
  id: string;
};

interface Props {
  id: string;
}

export default function TreeMenuItem({ id }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const selectedId = useEditorStore((state) => state.selectedId);
  const name = useEditorStore((state) => state.scene.entities[id].name);
  const children = useEditorStore((state) => state.scene.entities[id].children);
  const [open, setOpen] = useState(true);

  // Create drag source
  const [{ isDragging }, drag] = useDrag(
    () => ({
      type: DND_TYPES.Entity,
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
      accept: DND_TYPES.Entity,
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
      accept: DND_TYPES.Entity,
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
          useEditorStore.setState({ selectedId: id });
        }}
        className={`flex h-6 items-center rounded px-2 font-bold ${bgClass} ${opacityClass}`}
      >
        <div
          onClick={() => setOpen((prev) => !prev)}
          className="text-outline w-5 transition hover:text-inherit"
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
          className={`-mt-1 h-1 w-full rounded ${highlightBelowClass}`}
        />
      )}
    </div>
  );
}

function isDeepChild(entityId: string, parentId: string) {
  const { scene } = useEditorStore.getState();
  const entity = scene.entities[parentId];

  if (entity.children.includes(entityId)) return true;

  for (const child of entity.children) {
    if (isDeepChild(entityId, child)) return true;
  }

  return false;
}
