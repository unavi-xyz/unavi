import { useRef, useState } from "react";
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
  const ref = useRef<HTMLDivElement>(null);

  const tree = useStudioStore((state) => state.scene.tree);
  const selectedId = useStudioStore((state) => state.selectedId);
  const moveObject = useStudioStore((state) => state.moveObject);

  const [open, setOpen] = useState(true);

  const [{ isDragging }, drag] = useDrag(
    () => ({
      type: DND_TYPES.TreeObject,
      item: { id: object.id },
      collect: (monitor) => ({
        isDragging: monitor.isDragging(),
      }),
    }),
    [object]
  );

  const [{ isOver }, drop] = useDrop(
    () => ({
      accept: DND_TYPES.TreeObject,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== object.id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          const dropped = findObjectById(tree, droppedId);
          if (!dropped) return;

          //add as child
          moveObject(droppedId, object.id);
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
      }),
    }),
    [object, tree, moveObject]
  );

  const [{ isOver: isOverAbove }, dropAbove] = useDrop(
    () => ({
      accept: DND_TYPES.TreeObject,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== object.id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          const dropped = findObjectById(tree, droppedId);
          if (!dropped) return;

          if (!object.parentId) return;

          //add as sibling
          const parent = findObjectById(tree, object.parentId);
          if (!parent) return;

          const index = parent.children.indexOf(object);
          moveObject(droppedId, parent.id, index);
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
      }),
    }),
    [object, tree, moveObject]
  );

  const [{ isOver: isOverBelow }, dropBelow] = useDrop(
    () => ({
      accept: DND_TYPES.TreeObject,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== object.id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          const dropped = findObjectById(tree, droppedId);
          if (!dropped) return;

          if (!object.parentId) return;

          //add as sibling
          const parent = findObjectById(tree, object.parentId);
          if (!parent) return;

          moveObject(droppedId, object.id);
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
      }),
    }),
    [object, tree, moveObject]
  );

  const hasChildren = object.children.length > 0;
  const isSelected = selectedId === object.id;
  const bgClass = isSelected || isOver ? "bg-neutral-100" : "";
  const paddingClass = isRoot ? "" : "pl-4";
  const opacityClass = isDragging ? "opacity-0" : "";
  const highlightAboveClass = isOverAbove ? "bg-neutral-200" : "";
  const highlightBelowClass = isOverBelow ? "bg-neutral-200" : "";

  drop(ref);

  if (!isRoot) drag(ref);

  return (
    <div className="h-full relative">
      {!isRoot && (
        <div
          ref={dropAbove}
          className={`h-1.5 ml-4 rounded-full ${highlightAboveClass} ${paddingClass}`}
        />
      )}

      <div
        ref={ref}
        onClick={(e) => {
          e.stopPropagation();
          if (isRoot) useStudioStore.setState({ selectedId: undefined });
        }}
        className={`space-y-1 h-full ${paddingClass}`}
      >
        {!isRoot && (
          <div
            onClick={() => useStudioStore.setState({ selectedId: object.id })}
            className={`font-bold hover:bg-neutral-100 rounded-md px-2 cursor-pointer
                        flex items-center h-8 ${bgClass} ${opacityClass}`}
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

        <div>
          {open &&
            object.children.map((child, i) => {
              if (i === object.children.length - 1 && !isRoot) {
                return (
                  <div key={child.id}>
                    <TreeMenuItem object={child} />
                    <div
                      ref={dropBelow}
                      className={`h-1.5 ml-4 rounded-full ${highlightBelowClass} ${paddingClass}`}
                    />
                  </div>
                );
              }

              return <TreeMenuItem key={child.id} object={child} />;
            })}
        </div>
      </div>
    </div>
  );
}
