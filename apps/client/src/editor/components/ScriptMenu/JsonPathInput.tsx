import { Node } from "@gltf-transform/core";
import { parseJSONPath, ValueType } from "@unavi/gltf-extensions";
import { useEffect, useMemo, useState } from "react";

import { useNodes } from "../../hooks/useNodes";
import { useSubscribe } from "../../hooks/useSubscribe";
import { useEditor } from "../Editor";
import { FlowNodeParamter } from "./types";

interface Props {
  onChange: (key: string, value: FlowNodeParamter) => void;
  value?: string;
  pathType?: string;
}

export default function JsonPathInput({ onChange, value, pathType }: Props) {
  const pathOptions = useMemo(() => {
    if (!pathType) return ["Translation", "Rotation", "Scale"];
    if (pathType === ValueType.vec3) return ["Translation", "Scale"];
    if (pathType === ValueType.vec4) return ["Rotation"];
    if (pathType === ValueType.quat) return ["Rotation"];
    return [];
  }, [pathType]);

  const { mode } = useEditor();
  const nodes = useNodes();

  const [pathNode, setPathNode] = useState<Node>();
  const [pathProperty, setPathProperty] = useState("");

  useEffect(() => {
    setPathProperty(pathOptions[0]?.toLowerCase() ?? "");
  }, [pathOptions]);

  useEffect(() => {
    const path = parseJSONPath(value ?? "");

    if (!path) {
      setPathNode(nodes[0]);
      return;
    }

    const nodeId = nodes[Number(path.index)];
    setPathNode(nodeId);
    setPathProperty(path.property);
  }, [nodes, value]);

  useEffect(() => {
    if (!pathNode) return;
    const index = nodes.indexOf(pathNode);
    onChange("jsonPath", { value: `/nodes/${index}/${pathProperty}` });
  }, [pathNode, pathProperty, nodes, onChange]);

  const capitalizedProperty = pathProperty.charAt(0).toUpperCase() + pathProperty.slice(1);

  return (
    <div className="flex items-center space-x-1">
      <select
        value={pathNode ? nodes.indexOf(pathNode) : 0}
        disabled={mode === "play"}
        onChange={(e) => {
          const newNode = nodes[Number(e.currentTarget.value)];
          setPathNode(newNode);
        }}
        className={`h-6 rounded bg-neutral-200 px-1 ${
          mode === "play" ? "" : "hover:bg-neutral-300/80 focus:bg-neutral-300/80"
        }`}
      >
        {nodes.map((node, i) => (
          <NodeOption key={i} value={i} node={node} />
        ))}
      </select>

      <div>/</div>

      <select
        value={capitalizedProperty}
        disabled={mode === "play"}
        onChange={(e) => {
          setPathProperty(e.currentTarget.value.toLowerCase());
        }}
        className={`h-6 rounded bg-neutral-200 px-1 ${
          mode === "play" ? "" : "hover:bg-neutral-300/80 focus:bg-neutral-300/80"
        }`}
      >
        {pathOptions.map((option) => {
          return (
            <option key={option} value={option} className="text-lg">
              {option}
            </option>
          );
        })}
      </select>
    </div>
  );
}

function NodeOption({ node, value }: { node: Node; value: number }) {
  const name = useSubscribe(node, "Name");

  return (
    <option value={value} className="text-lg">
      {name}
    </option>
  );
}
