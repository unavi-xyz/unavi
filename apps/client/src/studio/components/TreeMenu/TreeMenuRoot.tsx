import { useRef } from "react";
import { useDrop } from "react-dnd";

import { useStudioStore } from "../../../studio/store";
import { DND_TYPES } from "../../../studio/types";
import { moveEntity } from "../../actions/MoveEntityAction";
import { getRootEntities } from "../../utils/tree";
import TreeMenuItem from "./TreeMenuItem";

type DragItem = {
  id: string;
};

export default function TreeMenuRoot() {
  const ref = useRef<HTMLDivElement>(null);
  const children = getRootEntities();

  // Create drop target
  const [{}, drop] = useDrop(
    () => ({
      accept: DND_TYPES.TreeItem,
      drop({ id: droppedId }: DragItem, monitor) {
        const didDrop = monitor.didDrop();
        if (didDrop) return;

        // Move to root
        moveEntity(droppedId, null);
      },
      collect: () => ({}),
    }),
    []
  );

  drop(ref);

  return (
    <div
      ref={ref}
      onMouseDown={() => useStudioStore.setState({ selectedId: null })}
      className="h-full"
    >
      <div>
        {children.map((child) => (
          <TreeMenuItem key={child.id} id={child.id} />
        ))}
      </div>
    </div>
  );
}
