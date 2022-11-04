import { useRef } from "react";
import { useDrop } from "react-dnd";

import { moveNode } from "../../actions/MoveNodeAction";
import { useNode } from "../../hooks/useNode";
import { useSubscribeValue } from "../../hooks/useSubscribeValue";
import { useEditorStore } from "../../store";
import { DND_TYPES } from "../../types";
import TreeMenuItem from "./TreeMenuItem";

type DragItem = {
  id: string;
};

export default function TreeMenuRoot() {
  const ref = useRef<HTMLDivElement>(null);
  const childrenIds$ = useNode("root", (node) => node.childrenIds$);
  const childrenIds = useSubscribeValue(childrenIds$);

  // Create drop target
  const [, drop] = useDrop(
    () => ({
      accept: DND_TYPES.Node,
      drop({ id: droppedId }: DragItem, monitor) {
        const didDrop = monitor.didDrop();
        if (didDrop) return;

        // Move to root
        moveNode(droppedId, "root");
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
