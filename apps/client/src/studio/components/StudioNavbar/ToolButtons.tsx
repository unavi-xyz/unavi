"use client";

import * as ToggleGroup from "@radix-ui/react-toggle-group";
import { BiMove } from "react-icons/bi";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import { MdSync } from "react-icons/md";

import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
import { Tool, useStudio } from "../Studio";

export default function ToolButtons() {
  const { loaded, tool, setTool } = useStudio();

  return (
    <ToggleGroup.Root
      type="single"
      value={tool}
      disabled={!loaded}
      onValueChange={(value: Tool | "") => {
        if (value) setTool(value);
      }}
      className="flex h-full w-full justify-center space-x-2"
    >
      <ToolButton tool="translate" tooltip="Translate">
        <BiMove />
      </ToolButton>

      <ToolButton tool="rotate" tooltip="Rotate">
        <MdSync />
      </ToolButton>

      <ToolButton tool="scale" tooltip="Scale">
        <CgArrowsExpandUpRight />
      </ToolButton>
    </ToggleGroup.Root>
  );
}

interface Props {
  tool: Tool;
  tooltip: string;
  children: React.ReactNode;
}

function ToolButton({ tool, tooltip, children }: Props) {
  const { tool: selectedTool, setTool } = useStudio();
  const selected = selectedTool === tool;

  return (
    <Tooltip text={tooltip} side="bottom">
      <ToggleGroup.Item value={tool} asChild>
        <IconButton selected={selected} onClick={() => setTool(tool)}>
          {children}
        </IconButton>
      </ToggleGroup.Item>
    </Tooltip>
  );
}
