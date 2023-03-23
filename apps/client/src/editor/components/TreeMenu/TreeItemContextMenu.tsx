import * as ContextMenu from "@radix-ui/react-context-menu";
import { deepDisposeNode } from "engine";
import { MdClose } from "react-icons/md";

import { useEditorStore } from "../../../../app/editor/[id]/store";

export default function TreeItemContextMenu() {
  function handleDelete() {
    const { engine, selectedId } = useEditorStore.getState();
    if (!engine || !selectedId) return;

    const node = engine.scene.node.store.get(selectedId);
    if (!node) throw new Error("Node not found");

    useEditorStore.setState({ selectedId: null });
    deepDisposeNode(node);
  }

  return (
    <ContextMenu.Portal>
      <ContextMenu.Content className="overflow-hidden rounded bg-white shadow">
        <ContextMenu.Item
          onClick={handleDelete}
          className="flex cursor-pointer select-none items-center space-x-2 rounded px-3 py-1 outline-none hover:bg-red-200 hover:text-red-900"
        >
          <MdClose className="text-lg" />
          <div>Delete</div>
        </ContextMenu.Item>
      </ContextMenu.Content>
    </ContextMenu.Portal>
  );
}
