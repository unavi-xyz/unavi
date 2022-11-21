import { useRef, useState } from "react";
import { useDrag, useDrop } from "react-dnd";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

import { moveNode } from "../../actions/MoveNodeAction";
import { useNode } from "../../hooks/useNode";
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

  const nodes$ = useEditorStore((state) => state.engine?.scene.nodes$);
  const name$ = useNode(id, (node) => node.name$);
  const childrenIds$ = useNode(id, (node) => node.childrenIds$);
  const isInternal$ = useNode(id, (node) => node.isInternal$);

  const name = useSubscribeValue(name$);
  const childrenIds = useSubscribeValue(childrenIds$);
  const isInternal = useSubscribeValue(isInternal$);
  const nodes = useSubscribeValue(nodes$);

  const children = nodes ? childrenIds?.map((id) => nodes[id]) : [];

  // Create drag source
  const [{ isDragging }, drag] = useDrag(
    () => ({
      type: DND_TYPES.Node,
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
      accept: DND_TYPES.Node,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== id) {
          const didDrop = monitor.didDrop();
          if (didDrop || isDeepChild(id, droppedId)) return;

          // Move to new parent
          moveNode(droppedId, id);
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
      accept: DND_TYPES.Node,
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

  const allChildrenInternal = children?.every((child) => child?.isInternal);
  const hasChildren =
    childrenIds && childrenIds.length > 0 && !allChildrenInternal;
  const isSelected = selectedId === id;
  const bgClass = isSelected
    ? "bg-primaryContainer text-onPrimaryContainer"
    : isOver
    ? "bg-surfaceVariant"
    : "hover:bg-surfaceVariant";
  const opacityClass = isDragging ? "opacity-0" : null;
  const highlightBelowClass = isOverBelow ? "bg-primary" : null;

  if (isInternal) return null;

  drop(ref);
  drag(ref);

  return (
    <div ref={ref} className="h-full">
      <div
        onMouseDown={(e) => {
          e.stopPropagation();
          useEditorStore.setState({ selectedId: id });
        }}
        className={`flex h-6 items-center rounded-md px-2 font-bold ${bgClass} ${opacityClass}`}
      >
        <div
          onClick={() => setOpen((prev) => !prev)}
          className="w-5 shrink-0 text-outline transition hover:text-inherit"
        >
          {hasChildren &&
            (open ? <IoMdArrowDropdown /> : <IoMdArrowDropright />)}
        </div>

        <div className="whitespace-nowrap">{name}</div>
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
          className={`-mt-1 h-1 w-full rounded-md ${highlightBelowClass}`}
        />
      )}
    </div>
  );
}

function isDeepChild(nodeId: string, parentId: string) {
  const node = useEditorStore.getState().getNode(parentId);
  if (!node) return false;

  if (node.childrenIds.includes(nodeId)) return true;

  for (const child of node.childrenIds) {
    if (isDeepChild(nodeId, child)) return true;
  }

  return false;
}
