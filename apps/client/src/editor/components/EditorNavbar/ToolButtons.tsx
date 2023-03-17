import * as ToggleGroup from "@radix-ui/react-toggle-group";
import React from "react";
import { BiMove } from "react-icons/bi";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import { MdSync } from "react-icons/md";

import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
import { useEditorStore } from "../../store";
import { Tool } from "../../types";

export default function ToolButtons() {
  const tool = useEditorStore((state) => state.tool);
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);

  return (
    <ToggleGroup.Root
      type="single"
      value={tool}
      disabled={!sceneLoaded}
      onValueChange={(value: Tool | "") => {
        if (value) useEditorStore.setState({ tool: value });
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
  const selected = useEditorStore((state) => state.tool === tool);

  return (
    <Tooltip text={tooltip} side="bottom">
      <ToggleGroup.Item value={tool} asChild>
        <IconButton selected={selected} onClick={() => useEditorStore.setState({ tool })}>
          {children}
        </IconButton>
      </ToggleGroup.Item>
    </Tooltip>
  );
}
