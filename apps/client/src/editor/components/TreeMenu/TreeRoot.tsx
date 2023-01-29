import { useEffect, useMemo } from "react";

import { useNodes } from "../../hooks/useNodes";
import { useEditorStore } from "../../store";
import TreeItem from "./TreeItem";

export default function TreeRoot() {
  const engine = useEditorStore((state) => state.engine);
  const treeIds = useEditorStore((state) => state.treeIds);
  const openIds = useEditorStore((state) => state.openIds);

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
      useEditorStore.setState({ draggingId: null });
    };

    window.addEventListener("mouseup", onMouseUp);
    return () => window.removeEventListener("mouseup", onMouseUp);
  }, []);

  useEffect(() => {
    if (!engine) return;

    const { treeIds, openIds } = useEditorStore.getState();
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

    if (!isSameTree) useEditorStore.setState({ treeIds: newTreeIds });
    if (!isSameOpen) useEditorStore.setState({ openIds: newOpenIds });
  }, [engine, nodeIds]);

  return (
    <div
      className="h-full pt-0.5"
      onMouseDown={() => useEditorStore.setState({ selectedId: null })}
    >
      {visibleIds.map((id) => {
        return <TreeItem key={id} id={id} />;
      })}
    </div>
  );
}
