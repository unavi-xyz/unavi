import * as ContextMenu from "@radix-ui/react-context-menu";
import { deepDisposeNode } from "@unavi/engine";
import { MdClose } from "react-icons/md";

import { useStudio } from "../Studio";

export default function TreeItemContextMenu() {
  const { engine, selectedId, setSelectedId } = useStudio();

  function handleDelete() {
    if (!engine || !selectedId) return;

    const node = engine.scene.node.store.get(selectedId);
    if (!node) throw new Error("Node not found");

    setSelectedId(null);
    deepDisposeNode(node);
  }

  return (
    <ContextMenu.Portal>
      <ContextMenu.Content className="rounded-lg bg-neutral-50 shadow">
        <ContextMenu.Item
          onClick={handleDelete}
          className="flex cursor-pointer select-none items-center space-x-2 rounded-lg px-3 py-1 outline-none hover:bg-red-200 hover:text-red-900"
        >
          <MdClose className="text-lg" />
          <div>Delete</div>
        </ContextMenu.Item>
      </ContextMenu.Content>
    </ContextMenu.Portal>
  );
}
