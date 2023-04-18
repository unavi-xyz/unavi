import * as ContextMenu from "@radix-ui/react-context-menu";
import { useEffect, useMemo } from "react";

import { useNodes } from "../../hooks/useNodes";
import { useEditor } from "../Editor";
import { useTree } from "./Tree";
import TreeItem from "./TreeItem";
import TreeItemContextMenu from "./TreeItemContextMenu";

export default function TreeRoot() {
  const { engine, mode, setSelectedId } = useEditor();
  const { openIds, setOpenIds, setTreeIds, setDraggingId, treeIds } = useTree();

  const nodes = useNodes();
  const nodeIds = useMemo(() => {
    const ids = nodes
      .map((node) => engine?.scene.node.getId(node))
      .filter((id) => id !== undefined) as string[];

    return ids;
  }, [nodes, engine]);

  const visibleIds = useMemo(() => {
    const isVisible = (id: string): boolean => {
      if (!engine) return false;

      const node = engine.scene.node.store.get(id);
      if (!node) throw new Error("Node not found");

      const parentId = engine.scene.node.getParent(node);
      if (!parentId) return true;

      const isParentOpen = openIds.includes(parentId);
      if (!isParentOpen) return false;

      return isVisible(parentId);
    };

    const visible = treeIds.filter(isVisible);

    return visible;
  }, [engine, treeIds, openIds]);

  useEffect(() => {
    const onMouseUp = () => {
      document.body.style.removeProperty("cursor");
      setDraggingId(null);
    };

    window.addEventListener("mouseup", onMouseUp);
    return () => window.removeEventListener("mouseup", onMouseUp);
  }, [setDraggingId]);

  useEffect(() => {
    if (!engine) return;

    const newTreeIds = [...treeIds];
    const newOpenIds = [...openIds];

    // Add new nodes to tree
    const newIds = nodeIds.filter((id) => !treeIds.includes(id));

    // Move children after parents
    newIds.forEach((id) => {
      const node = engine.scene.node.store.get(id);
      if (!node) throw new Error("Node not found");

      function deepChildrenIds(id: string): string[] {
        if (!engine) return [];

        const node = engine.scene.node.store.get(id);
        if (!node) throw new Error("Node not found");

        const childrenIds = node.listChildren().map((child) => {
          const id = engine.scene.node.getId(child);
          if (!id) throw new Error("Node not found");
          return id;
        });

        return childrenIds.flatMap((id) => [id, ...deepChildrenIds(id)]);
      }

      // Get children ids
      const childrenIds = deepChildrenIds(id);

      // Remove children from newIds
      childrenIds.forEach((childId) => {
        const index = newIds.indexOf(childId);
        newIds.splice(index, 1);
      });

      // Add children after parent
      const index = newIds.indexOf(id);
      newIds.splice(index + 1, 0, ...childrenIds);
    });

    newTreeIds.push(...newIds);

    // Remove deleted nodes from tree
    const removedIds = newTreeIds.filter((id) => !nodeIds.includes(id));
    removedIds.forEach((id) => {
      const index = newTreeIds.indexOf(id);
      newTreeIds.splice(index, 1);

      const openIndex = newOpenIds.indexOf(id);
      newOpenIds.splice(openIndex, 1);
    });

    // Save changes
    const isSameTree =
      newTreeIds.every((id, index) => treeIds[index] === id) &&
      newTreeIds.length === treeIds.length;
    const isSameOpen =
      newOpenIds.every((id, index) => openIds[index] === id) &&
      newOpenIds.length === openIds.length;

    if (!isSameTree) setTreeIds(newTreeIds);
    if (!isSameOpen) setOpenIds(newOpenIds);

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [engine, nodeIds, setTreeIds, setOpenIds]);

  return (
    <ContextMenu.Root>
      <div onMouseDown={() => setSelectedId(null)} className="h-full">
        <ContextMenu.Trigger disabled={mode === "play"}>
          {visibleIds.map((id) => {
            return <TreeItem key={id} id={id} />;
          })}
        </ContextMenu.Trigger>
      </div>

      <TreeItemContextMenu />
    </ContextMenu.Root>
  );
}
