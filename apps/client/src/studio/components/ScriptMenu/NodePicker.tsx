import * as ContextMenu from "@radix-ui/react-context-menu";
import { IoIosArrowForward } from "react-icons/io";
import { XYPosition } from "reactflow";

import { categories, categorizedNodes } from "./nodes";
import { useScript } from "./Script";

interface Props {
  position?: XYPosition;
}

export default function NodePicker({ position }: Props) {
  const { addNode } = useScript();

  if (!position) return null;

  return (
    <ContextMenu.Portal>
      <ContextMenu.Content className="rounded-xl bg-neutral-50 shadow-md">
        <div className="py-2">
          {categories.map((category) => (
            <ContextMenu.Sub key={category}>
              <ContextMenu.SubTrigger className="flex cursor-default items-center justify-between space-x-3 pl-6 pr-3 capitalize focus:bg-neutral-200 focus:outline-none">
                <div>{category}</div>
                <IoIosArrowForward className="text-neutral-500" />
              </ContextMenu.SubTrigger>

              <ContextMenu.SubContent sideOffset={4} className="rounded-xl bg-neutral-50 shadow-md">
                <div className="max-h-96 overflow-y-auto py-2">
                  {categorizedNodes[category]?.map((node) => {
                    // Remove category from node type
                    const splitType = node.type.split("/");
                    splitType.shift();

                    const name = splitType.join("/");

                    return (
                      <ContextMenu.Item
                        key={node.type}
                        onClick={() => addNode(node.type, position)}
                        className="cursor-default px-6 capitalize focus:bg-neutral-200 focus:outline-none"
                      >
                        {name}
                      </ContextMenu.Item>
                    );
                  })}
                </div>
              </ContextMenu.SubContent>
            </ContextMenu.Sub>
          ))}
        </div>
      </ContextMenu.Content>
    </ContextMenu.Portal>
  );
}
