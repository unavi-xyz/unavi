import * as ContextMenu from "@radix-ui/react-context-menu";
import { MdClose } from "react-icons/md";

interface Props {
  onDelete: () => void;
}

export default function NodeContextMenu({ onDelete }: Props) {
  return (
    <ContextMenu.Portal>
      <ContextMenu.Content className="rounded-lg bg-neutral-50 shadow">
        <ContextMenu.Item
          onClick={onDelete}
          className="flex cursor-pointer select-none items-center space-x-2 rounded-lg px-3 py-1 outline-none hover:bg-red-200 hover:text-red-900"
        >
          <MdClose className="text-lg" />
          <div>Delete</div>
        </ContextMenu.Item>
      </ContextMenu.Content>
    </ContextMenu.Portal>
  );
}
