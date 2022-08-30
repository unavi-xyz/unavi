import { Not, defineQuery } from "bitecs";
import { useRef, useState } from "react";
import { useDrag, useDrop } from "react-dnd";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

import { Child, SceneObject, childObjectsQuery } from "@wired-xr/engine";

import { useStudioStore } from "../../../studio/store";
import { DND_TYPES } from "../../../studio/types";
import { addItemAsSibling, moveObject, updateTree } from "../../utils/tree";

type DragItem = {
  id: string;
};

interface Props {
  id?: number;
}

const ROOT = -1;

const rootObjectQuery = defineQuery([SceneObject, Not(Child)]);

export default function TreeMenuItem({ id = ROOT }: Props) {
  const isRoot = id === ROOT;

  const ref = useRef<HTMLDivElement>(null);
  const selectedId = useStudioStore((state) => state.selectedId);
  const engine = useStudioStore((state) => state.engine);
  const names = useStudioStore((state) => state.names);
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
        if (Number(droppedId) !== id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          // Move to new parent
          moveObject(Number(droppedId), id);
          updateTree();
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
        if (Number(droppedId) !== id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          // Add as sibling
          addItemAsSibling(Number(droppedId), id, "above");
          updateTree();
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
        if (Number(droppedId) !== id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          // Add as sibling
          addItemAsSibling(Number(droppedId), id, "below");
          updateTree();
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
      }),
    }),
    [id]
  );

  if (id == undefined || !engine) return null;

  const childObjects = childObjectsQuery(engine.world);
  const objectChildren = childObjects.filter((eid) => Child.parent[eid] === id);
  const rootObjects = rootObjectQuery(engine.world);
  const children = isRoot ? rootObjects : objectChildren;

  const hasChildren = children.length > 1;
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

            <div>{names[SceneObject.name[id]]}</div>
          </div>
        )}

        {open && (
          <div className={`${opacityClass}`}>
            {children.map((child, i) => {
              if (i === children.length - 1 && !isRoot) {
                return (
                  <div key={child} className="h-full">
                    <TreeMenuItem id={child} />
                    <div
                      ref={dropBelow}
                      className={`h-1 ml-4 rounded-full ${highlightBelowClass} ${marginClass}`}
                    />
                  </div>
                );
              }

              return <TreeMenuItem key={child} id={child} />;
            })}
          </div>
        )}
      </div>
    </div>
  );
}
