import { useMemo } from "react";
import { HiOutlineCube } from "react-icons/hi";
import { IoMdArrowDropdown, IoMdArrowDropright } from "react-icons/io";

import { useNodeAttribute } from "../../hooks/useNodeAttribute";
import { useEditorStore } from "../../store";
import { isAncestor } from "./utils/isAncestor";
import { moveNode } from "./utils/moveNode";

interface Props {
  id: string;
}

export default function TreeItem({ id }: Props) {
  const engine = useEditorStore((state) => state.engine);
  const draggingId = useEditorStore((state) => state.draggingId);
  const name = useNodeAttribute(id, "name");
  const childrenIds = useNodeAttribute(id, "children") ?? [];
  const selectedId = useEditorStore((state) => state.selectedId);
  const openIds = useEditorStore((state) => state.openIds);
  const treeIds = useEditorStore((state) => state.treeIds);

  const depth = useMemo(() => {
    const getDepth = (id: string): number => {
      if (!engine) return 0;

      const node = engine.scene.node.store.get(id);
      if (!node) throw new Error("Node not found");

      const parentId = engine.scene.node.getParent(node);
      if (!parentId) return 0;

      return getDepth(parentId) + 1;
    };

    return getDepth(id);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [engine, id, treeIds]);

  const isOpen = openIds.includes(id);
  const isSelected = selectedId === id;
  const hasChildren = childrenIds.length > 0;

  return (
    <div className="relative select-none">
      <div
        style={{ paddingLeft: `${depth + 1}rem` }}
        className={`flex items-center space-x-1 text-sm text-neutral-800 ${
          isSelected ? "bg-sky-200 text-black" : "hover:bg-neutral-200"
        }`}
        onMouseDown={(e) => {
          if (e.button !== 0) return;
          e.stopPropagation();
          useEditorStore.setState({ draggingId: id, selectedId: id });
          document.body.style.cursor = "grabbing";
        }}
        onMouseUp={(e) => {
          if (
            e.button !== 0 ||
            !engine ||
            !draggingId ||
            draggingId === id ||
            isAncestor(draggingId, id, engine)
          )
            return;

          const draggedNode = engine.scene.node.store.get(draggingId);
          if (!draggedNode) throw new Error("Node not found");

          const node = engine.scene.node.store.get(id);
          if (!node) throw new Error("Node not found");

          // Add dragged node as child
          const draggedParentId = engine.scene.node.getParent(draggedNode);
          if (draggedParentId !== id) {
            node.addChild(draggedNode);
          }

          // Move dragged node
          const targetIndex = treeIds.indexOf(id);
          moveNode(draggingId, targetIndex + 1);

          // Open children
          const alreadyOpen = openIds.includes(id);
          if (!alreadyOpen) useEditorStore.setState({ openIds: [...openIds, id] });
        }}
      >
        <div
          className={`w-3 shrink-0 hover:text-neutral-500 ${hasChildren && "cursor-pointer"}`}
          onMouseDown={(e) => {
            if (!hasChildren) return;

            e.stopPropagation();

            if (isOpen) {
              useEditorStore.setState({ openIds: openIds.filter((x) => x !== id) });
            } else {
              useEditorStore.setState({ openIds: [...openIds, id] });
            }
          }}
        >
          {hasChildren && (isOpen ? <IoMdArrowDropdown /> : <IoMdArrowDropright />)}
        </div>

        <HiOutlineCube className="shrink-0 text-lg" />

        <div className="overflow-x-hidden text-ellipsis whitespace-nowrap">{name}</div>
      </div>

      {draggingId && draggingId !== id ? (
        <div
          style={{ paddingLeft: `${depth + 2}rem` }}
          className="absolute left-0 -top-0.5 h-2 w-full opacity-0 hover:opacity-100"
          onMouseUp={() => {
            if (!engine || !draggingId || draggingId === id || isAncestor(draggingId, id, engine))
              return;

            const draggedNode = engine.scene.node.store.get(draggingId);
            if (!draggedNode) throw new Error("Node not found");

            const node = engine.scene.node.store.get(id);
            if (!node) throw new Error("Node not found");

            const parentId = engine.scene.node.getParent(node);
            const draggedParentId = engine.scene.node.getParent(draggedNode);

            if (parentId !== draggedParentId) {
              // Add dragged node as sibling
              if (parentId) {
                const parentNode = engine.scene.node.store.get(parentId);
                if (!parentNode) throw new Error("Parent node not found");

                parentNode.addChild(draggedNode);
              } else {
                if (draggedParentId) {
                  const draggedParent = engine.scene.node.store.get(draggedParentId);
                  if (!draggedParent) throw new Error("Dragged node parent not found");

                  draggedParent.removeChild(draggedNode);
                }
              }
            }

            // Move dragged node
            const targetIndex = treeIds.indexOf(id);
            moveNode(draggingId, targetIndex);
          }}
        >
          <div className="flex items-center">
            <div className="h-1 w-1 rounded-full bg-blue-400" />
            <div className="h-0.5 w-full bg-blue-400" />
          </div>
        </div>
      ) : null}
    </div>
  );
}
