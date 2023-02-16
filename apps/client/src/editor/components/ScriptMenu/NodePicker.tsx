import { NodeSpecJSON } from "@behave-graph/core";
import * as ContextMenu from "@radix-ui/react-context-menu";
import { useEffect, useRef, useState } from "react";
import { useReactFlow, XYPosition } from "reactflow";

import rawSpecJson from "./node-spec.json";

const usedNodes = rawSpecJson as NodeSpecJSON[];

interface Props {
  position?: XYPosition;
  addNode: (type: string, position: XYPosition) => void;
}

export default function NodePicker({ position, addNode }: Props) {
  const filterInputRef = useRef<HTMLInputElement>(null);
  const [filterString, setFilterString] = useState("");

  useEffect(() => {
    setFilterString("");
  }, [position]);

  const instance = useReactFlow();

  const filteredNodes = usedNodes.filter((node) =>
    node.type.toLowerCase().includes(filterString.toLowerCase())
  );

  if (!position) return null;

  return (
    <ContextMenu.Portal>
      <ContextMenu.Content className="overflow-hidden rounded bg-white shadow">
        <ContextMenu.Label className="bg-neutral-500 bg-gradient-to-t from-black/10 to-white/10 px-3 py-0.5 text-white">
          Add Node
        </ContextMenu.Label>

        <div className="space-y-2 p-2">
          <input
            ref={filterInputRef}
            type="text"
            autoFocus
            placeholder="Type to filter..."
            onKeyDown={(e) => e.stopPropagation()}
            onChange={(e) => setFilterString(e.target.value)}
            className="w-full rounded-full bg-neutral-200 pl-4 leading-8 transition placeholder:text-neutral-600"
          />

          <div className="max-h-48 overflow-y-auto">
            {filteredNodes.map(({ type }) => (
              <ContextMenu.Item
                key={type}
                onClick={() => {
                  if (position) addNode(type, instance.project(position));
                }}
                className="select-none rounded px-4 py-0.5 outline-none hover:bg-neutral-200"
              >
                {type}
              </ContextMenu.Item>
            ))}
          </div>
        </div>
      </ContextMenu.Content>
    </ContextMenu.Portal>
  );
}
