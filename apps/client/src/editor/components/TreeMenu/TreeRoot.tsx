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
      .map((node) => engine?.modules.scene.node.getId(node))
      .filter((id) => id !== undefined) as string[];

    return ids;
  }, [nodes, engine]);

  const visibleIds = useMemo(() => {
    const isVisible = (id: string): boolean => {
      if (!engine) return false;

      const node = engine.modules.scene.node.store.get(id);
      if (!node) throw new Error("Node not found");

      const parentId = engine.modules.scene.node.getParent(node);
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
      useEditorStore.setState({ draggingId: null });
    };

    window.addEventListener("mouseup", onMouseUp);
    return () => window.removeEventListener("mouseup", onMouseUp);
  }, []);

  useEffect(() => {
    if (!engine) return;

    const newTreeIds = [...treeIds];
    const newOpenIds = [...openIds];

    // Add new nodes to tree
    const newIds = nodeIds.filter((id) => !treeIds.includes(id));
    newIds.forEach((id) => newTreeIds.push(id));

    // Remove deleted nodes from tree
    const removedIds = newTreeIds.filter((id) => !nodeIds.includes(id));
    removedIds.forEach((id) => {
      const index = newTreeIds.indexOf(id);
      newTreeIds.splice(index, 1);

      const openIndex = newOpenIds.indexOf(id);
      newOpenIds.splice(openIndex, 1);
    });

    // Move children after parents
    newTreeIds.sort((a, b) => {
      if (!engine) return 0;

      const nodeA = engine.modules.scene.node.store.get(a);
      if (!nodeA) throw new Error("Node not found");

      const nodeB = engine.modules.scene.node.store.get(b);
      if (!nodeB) throw new Error("Node not found");

      const parentA = engine.modules.scene.node.getParent(nodeA);
      const parentB = engine.modules.scene.node.getParent(nodeB);

      if (!parentA) return -1;
      if (!parentB) return 1;

      if (parentA === b) return 1;
      if (parentB === a) return -1;

      return 0;
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
  }, [engine, nodeIds, treeIds, openIds]);

  return (
    <div className="h-full" onMouseDown={() => useEditorStore.setState({ selectedId: null })}>
      {visibleIds.map((id) => {
        return <TreeItem key={id} id={id} />;
      })}
    </div>
  );
}
