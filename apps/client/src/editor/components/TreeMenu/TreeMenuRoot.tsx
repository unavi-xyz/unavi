import { useRef } from "react";
import { useDrop } from "react-dnd";

import { moveEntity } from "../../actions/MoveEntityAction";
import { useEntity } from "../../hooks/useEntity";
import { useEditorStore } from "../../store";
import { DND_TYPES } from "../../types";
import TreeMenuItem from "./TreeMenuItem";

type DragItem = {
  id: string;
};

export default function TreeMenuRoot() {
  const ref = useRef<HTMLDivElement>(null);
  const childrenIds = useEntity("root", (entity) => entity.childrenIds);

  // Create drop target
  const [{}, drop] = useDrop(
    () => ({
      accept: DND_TYPES.Entity,
      drop({ id: droppedId }: DragItem, monitor) {
        const didDrop = monitor.didDrop();
        if (didDrop) return;

        // Move to root
        moveEntity(droppedId, "root");
      },
      collect: () => ({}),
    }),
    []
  );

  drop(ref);

  return (
    <div
      ref={ref}
      onMouseDown={() => useEditorStore.setState({ selectedId: null })}
      className="h-full"
    >
      <div>
        {childrenIds?.map((childId) => (
          <TreeMenuItem key={childId} id={childId} />
        ))}
      </div>
    </div>
  );
}
