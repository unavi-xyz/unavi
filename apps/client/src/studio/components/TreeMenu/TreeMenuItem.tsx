import { useRef, useState } from "react";
import { useDrag, useDrop } from "react-dnd";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";
import { Object3D } from "three";

import { useStudioStore } from "../../../studio/store";
import { DND_TYPES, UserData } from "../../../studio/types";
import { addObjectAsSibling, addObjectToParent, findObject } from "../../utils/scene";

type DragItem = {
  id: string;
};

interface Props {
  object: Object3D;
  isRoot?: boolean;
}

export default function TreeMenuItem({ object, isRoot = false }: Props) {
  const ref = useRef<HTMLDivElement>(null);

  const id = object.uuid;

  const selectedId = useStudioStore((state) => state.selectedId);

  const [open, setOpen] = useState(true);

  // Create drag source
  const [{ isDragging }, drag] = useDrag(
    () => ({
      type: DND_TYPES.TreeNode,
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
      accept: DND_TYPES.TreeNode,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          const dropped = findObject(droppedId);
          if (!dropped) return;

          // Move to new parent
          addObjectToParent(dropped, object);
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
      }),
    }),
    [id]
  );

  // Create drop target for above
  const [{ isOver: isOverAbove }, dropAbove] = useDrop(
    () => ({
      accept: DND_TYPES.TreeNode,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          const dropped = findObject(droppedId);
          if (!dropped || !object.parent) return;

          // Add as sibling
          addObjectAsSibling(dropped, object, "above");
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
      }),
    }),
    [id]
  );

  // Create drop target for below
  const [{ isOver: isOverBelow }, dropBelow] = useDrop(
    () => ({
      accept: DND_TYPES.TreeNode,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          const dropped = findObject(droppedId);
          if (!dropped || !object.parent) return;

          // Add as sibling
          addObjectAsSibling(dropped, object, "below");
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
      }),
    }),
    [id]
  );

  if (!object) return null;

  const hasChildren = object.children.length > 0;
  const isSelected = selectedId === id;
  const bgClass =
    isSelected || isOver
      ? "bg-primaryContainer text-onPrimaryContainer"
      : "hover:bg-surfaceVariant";
  const marginClass = isRoot ? "" : "ml-4";
  const opacityClass = isDragging ? "opacity-0" : "";
  const highlightAboveClass = isOverAbove ? "bg-primaryContainer" : "";
  const highlightBelowClass = isOverBelow ? "bg-primaryContainer" : "";

  drop(ref);

  if (!isRoot) drag(ref);

  return (
    <div className="h-full relative">
      {!isRoot && (
        <div
          ref={dropAbove}
          className={`h-1 ml-4 rounded-full ${highlightAboveClass} ${marginClass}`}
        />
      )}

      <div
        ref={ref}
        onMouseDown={() => {
          if (isRoot) useStudioStore.setState({ selectedId: null });
        }}
        className={`h-full ${marginClass}`}
      >
        {!isRoot && (
          <div
            onMouseDown={(e) => {
              e.stopPropagation();
              useStudioStore.setState({ selectedId: id });
            }}
            className={`font-bold rounded-md px-2 flex items-center
                        h-6 ${bgClass} ${opacityClass}`}
          >
            <div
              onClick={() => setOpen((prev) => !prev)}
              className="w-5 text-outline hover:text-inherit transition"
            >
              {hasChildren && (open ? <IoMdArrowDropdown /> : <IoMdArrowDropright />)}
            </div>

            <div>{object.name ?? object.type}</div>
          </div>
        )}

        {open && (
          <div className={`${opacityClass}`}>
            {object.children.map((child, i) => {
              const userData: UserData = child.userData;

              // If not a node, skip
              if (!userData.treeNode) return null;

              if (i === object.children.length - 1 && !isRoot) {
                return (
                  <div key={child.id} className="h-full">
                    <TreeMenuItem object={child} />
                    <div
                      ref={dropBelow}
                      className={`h-1 ml-4 rounded-full ${highlightBelowClass} ${marginClass}`}
                    />
                  </div>
                );
              }

              return <TreeMenuItem key={child.id} object={child} />;
            })}
          </div>
        )}
      </div>
    </div>
  );
}
