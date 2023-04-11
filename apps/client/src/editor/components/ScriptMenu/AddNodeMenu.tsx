import { IoIosArrowForward } from "react-icons/io";

import {
  DropdownContent,
  DropdownItem,
  DropdownSub,
  DropdownSubContent,
  DropdownSubTrigger,
} from "../../../ui/DropdownMenu";
import { useEditor } from "../Editor";
import { categories, categorizedNodes } from "./nodes";
import { useScript } from "./Script";

export default function AddNodeMenu() {
  const { engine } = useEditor();
  const { addNode, reactflow } = useScript();

  return (
    <DropdownContent>
      <div className="py-2">
        {categories.map((category) => (
          <DropdownSub key={category}>
            <DropdownSubTrigger className="flex cursor-default items-center justify-between space-x-3 pl-6 pr-3 capitalize focus:bg-neutral-200 focus:outline-none">
              <div>{category}</div>
              <IoIosArrowForward className="text-neutral-500" />
            </DropdownSubTrigger>

            <DropdownSubContent>
              <div className="max-h-96 overflow-y-auto py-2">
                {categorizedNodes[category]?.map((node) => {
                  // Remove category from node type
                  const splitType = node.type.split("/");
                  splitType.shift();

                  const name = splitType.join("/");

                  return (
                    <DropdownItem
                      key={node.type}
                      onClick={() => {
                        if (!engine || !reactflow) return;
                        addNode(node.type, reactflow.project({ x: window.innerWidth / 3, y: 40 }));
                      }}
                      className="cursor-default px-6 capitalize focus:bg-neutral-200 focus:outline-none"
                    >
                      {name}
                    </DropdownItem>
                  );
                })}
              </div>
            </DropdownSubContent>
          </DropdownSub>
        ))}
      </div>
    </DropdownContent>
  );
}
