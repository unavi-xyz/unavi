import { useRef } from "react";
import { useDrop } from "react-dnd";

import { moveEntity } from "../../actions/MoveEntityAction";
import { useEditorStore } from "../../store";
import { DND_TYPES } from "../../types";
import TreeMenuItem from "./TreeMenuItem";

type DragItem = {
  id: string;
};

export default function TreeMenuRoot() {
  const ref = useRef<HTMLDivElement>(null);
  const children = useEditorStore((state) => state.tree["root"].children);

  // Create drop target
  const [{}, drop] = useDrop(
    () => ({
      accept: DND_TYPES.TreeItem,
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
        {children.map((child) => (
          <TreeMenuItem key={child} id={child} />
        ))}
      </div>
    </div>
  );
}
