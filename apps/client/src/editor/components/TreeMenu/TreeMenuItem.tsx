import { useRef, useState } from "react";
import { useDrag, useDrop } from "react-dnd";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

import { moveEntity } from "../../actions/MoveEntityAction";
import { useEntity } from "../../hooks/useEntity";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { useEditorStore } from "../../store";
import { DND_TYPES } from "../../types";
import { moveToSibling } from "../../utils/moveToSibling";

type DragItem = {
  id: string;
};

interface Props {
  id: string;
}

export default function TreeMenuItem({ id }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(true);
  const selectedId = useEditorStore((state) => state.selectedId);

  const name$ = useEntity(id, (entity) => entity.name$);
  const childrenIds$ = useEntity(id, (entity) => entity.childrenIds$);

  const name = useSubscribeValue(name$);
  const childrenIds = useSubscribeValue(childrenIds$);

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
          if (didDrop || isDeepChild(id, droppedId)) return;

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
          if (didDrop || isDeepChild(id, droppedId)) return;

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

  const hasChildren = childrenIds && childrenIds.length > 0;
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
          className="w-5 text-outline transition hover:text-inherit"
        >
          {hasChildren &&
            (open ? <IoMdArrowDropdown /> : <IoMdArrowDropright />)}
        </div>

        <div>{name}</div>
      </div>

      {open && hasChildren && (
        <div className={`pl-4 ${opacityClass}`}>
          {childrenIds.map((childId) => (
            <TreeMenuItem key={childId} id={childId} />
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
  const entity = useEditorStore.getState().getEntity(parentId);
  if (!entity) return false;

  if (entity.childrenIds.includes(entityId)) return true;

  for (const child of entity.childrenIds) {
    if (isDeepChild(entityId, child)) return true;
  }

  return false;
}
