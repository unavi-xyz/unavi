import * as ToggleGroup from "@radix-ui/react-toggle-group";
import { Tool, useSceneStore } from "@unavi/engine";
import { BiMove } from "react-icons/bi";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import { MdSync } from "react-icons/md";

import Tooltip from "@/src/ui/Tooltip";

export default function Tools() {
  const tool = useSceneStore((state) => state.tool);

  return (
    <div className="fixed bottom-0 left-1/2 z-20 -translate-x-1/2 p-4">
      <ToggleGroup.Root
        type="single"
        value={tool}
        defaultValue={tool}
        onValueChange={(value) => {
          useSceneStore.setState({ tool: value as Tool });
        }}
        className="flex items-center space-x-1 rounded-lg bg-neutral-900 p-1 text-white"
      >
        <ToolButton tool={Tool.Translate}>
          <BiMove className="text-2xl" />
        </ToolButton>

        <ToolButton tool={Tool.Rotate}>
          <MdSync className="text-2xl" />
        </ToolButton>

        <ToolButton tool={Tool.Scale}>
          <CgArrowsExpandUpRight className="text-2xl" />
        </ToolButton>
      </ToggleGroup.Root>
    </div>
  );
}

interface ToolButtonProps {
  tool: Tool;
  children: React.ReactNode;
}

function ToolButton({ tool, children }: ToolButtonProps) {
  const selected = useSceneStore((state) => state.tool);
  const isSelected = selected === tool;

  return (
    <Tooltip text={tool} side="top">
      <ToggleGroup.Item
        value={tool}
        className={`flex items-center justify-center rounded-md p-1.5 transition active:opacity-90 ${
          isSelected
            ? "bg-neutral-700/70 hover:bg-neutral-700"
            : "hover:bg-neutral-700/70"
        }`}
      >
        {children}
      </ToggleGroup.Item>
    </Tooltip>
  );
}
