import { useRef, useState } from "react";
import { useDrag, useDrop } from "react-dnd";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

import { IEntity, findEntityById } from "@wired-xr/engine";

import { useStudioStore } from "../../../studio/store";
import { DND_TYPES } from "../../../studio/types";

type DragItem = {
  id: string;
};

interface Props {
  entity: IEntity;
  isRoot?: boolean;
}

export default function TreeMenuItem({ entity, isRoot = false }: Props) {
  const ref = useRef<HTMLDivElement>(null);

  const tree = useStudioStore((state) => state.scene.tree);
  const selectedId = useStudioStore((state) => state.selectedId);
  const moveEntity = useStudioStore((state) => state.moveEntity);

  const [open, setOpen] = useState(true);

  const [{ isDragging }, drag] = useDrag(
    () => ({
      type: DND_TYPES.Entity,
      item: { id: entity.id },
      collect: (monitor) => ({
        isDragging: monitor.isDragging(),
      }),
    }),
    [entity]
  );

  const [{ isOver }, drop] = useDrop(
    () => ({
      accept: DND_TYPES.Entity,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== entity.id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          const dropped = findEntityById(tree, droppedId);
          if (!dropped) return;

          //add as child
          moveEntity(droppedId, entity.id);
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
      }),
    }),
    [entity, tree, moveEntity]
  );

  const [{ isOver: isOverAbove }, dropAbove] = useDrop(
    () => ({
      accept: DND_TYPES.Entity,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== entity.id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          const dropped = findEntityById(tree, droppedId);
          if (!dropped) return;

          if (!entity.parentId) return;

          //add as sibling
          const parent = findEntityById(tree, entity.parentId);
          if (!parent) return;

          const index = parent.children.indexOf(entity);
          moveEntity(droppedId, parent.id, index);
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
      }),
    }),
    [entity, tree, moveEntity]
  );

  const [{ isOver: isOverBelow }, dropBelow] = useDrop(
    () => ({
      accept: DND_TYPES.Entity,
      drop({ id: droppedId }: DragItem, monitor) {
        if (droppedId !== entity.id) {
          const didDrop = monitor.didDrop();
          if (didDrop) return;

          const dropped = findEntityById(tree, droppedId);
          if (!dropped) return;

          if (!entity.parentId) return;

          //add as sibling
          const parent = findEntityById(tree, entity.parentId);
          if (!parent) return;

          moveEntity(droppedId, entity.id);
        }
      },
      collect: (monitor) => ({
        isOver: monitor.isOver(),
      }),
    }),
    [entity, tree, moveEntity]
  );

  const hasChildren = entity.children.length > 0;
  const isSelected = selectedId === entity.id;
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
          if (isRoot) useStudioStore.setState({ selectedId: undefined });
        }}
        className={`h-full ${marginClass}`}
      >
        {!isRoot && (
          <div
            onMouseDown={(e) => {
              e.stopPropagation();
              useStudioStore.setState({ selectedId: entity.id });
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

            <div>{entity.name}</div>
          </div>
        )}

        {open && (
          <div className={`${opacityClass}`}>
            {entity.children.map((child, i) => {
              if (i === entity.children.length - 1 && !isRoot) {
                return (
                  <div key={child.id} className="h-full">
                    <TreeMenuItem entity={child} />
                    <div
                      ref={dropBelow}
                      className={`h-1 ml-4 rounded-full ${highlightBelowClass} ${marginClass}`}
                    />
                  </div>
                );
              }

              return <TreeMenuItem key={child.id} entity={child} />;
            })}
          </div>
        )}
      </div>
    </div>
  );
}
