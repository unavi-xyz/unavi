import * as ContextMenu from "@radix-ui/react-context-menu";
import { nanoid } from "nanoid";
import { useEffect, useRef, useState } from "react";
import { useReactFlow, XYPosition } from "reactflow";

import { useEditorStore } from "../../store";
import { getNodeSpecJSON } from "./utils/getNodeSpecJSON";

const allNodes = getNodeSpecJSON();

const hiddenNodes = [
  "scene/get/boolean",
  "scene/get/float",
  "scene/get/string",
  "scene/get/vec2",
  "scene/set/boolean",
  "scene/set/float",
  "scene/set/string",
  "scene/set/vec2",
];

const usedNodes = allNodes.filter((node) => {
  const type = node.type.toLowerCase();

  if (type.includes("color")) return false;
  if (type.includes("euler")) return false;
  if (type.includes("integer")) return false;
  if (type.includes("vec4")) return false;
  if (type.includes("mat3")) return false;
  if (type.includes("mat4")) return false;
  if (type.includes("customevent")) return false;

  if (hiddenNodes.includes(node.type)) return false;

  return true;
});

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

          <div className="max-h-56 overflow-y-auto">
            {filteredNodes.map(({ type }) => (
              <ContextMenu.Item
                key={type}
                onClick={() => {
                  const { engine } = useEditorStore.getState();
                  if (!engine) return;

                  if (type.includes("variable")) {
                    // If no variables, create one
                    const { variables } = useEditorStore.getState();
                    if (variables.length === 0) {
                      const newVariable = engine.scene.extensions.behavior.createVariable();
                      newVariable.setName(nanoid(8));
                      useEditorStore.setState({ variables: [...variables, newVariable] });
                    }
                  }

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
