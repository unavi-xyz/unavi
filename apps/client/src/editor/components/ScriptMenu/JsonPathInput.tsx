import { parseJSONPath, ValueType } from "engine";
import { useEffect, useMemo, useState } from "react";

import { useNodeAttribute } from "../../hooks/useNodeAttribute";
import { useNodes } from "../../hooks/useNodes";
import { useEditorStore } from "../../store";

interface Props {
  onChange: (key: string, value: any) => void;
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

  const engine = useEditorStore((state) => state.engine);
  const nodes = useNodes();

  const nodeIds = useMemo(() => {
    const ids = nodes
      .map((node) => engine?.scene.node.getId(node))
      .filter((id) => id !== undefined) as string[];

    return ids;
  }, [nodes, engine]);

  const [pathNode, setPathNode] = useState<string>();
  const [pathProperty, setPathProperty] = useState("");

  useEffect(() => {
    setPathProperty(pathOptions[0]?.toLowerCase() ?? "");
  }, [pathOptions]);

  useEffect(() => {
    const path = parseJSONPath(value ?? "");

    if (!path) {
      setPathNode(nodeIds[0]);
      return;
    }

    const nodeId = nodeIds[Number(path.index)];
    setPathNode(nodeId);
    setPathProperty(path.property);
  }, [nodeIds, value]);

  useEffect(() => {
    if (!pathNode) return;
    const index = nodeIds.indexOf(pathNode);
    onChange("jsonPath", `/nodes/${index}/${pathProperty}`);
  }, [pathNode, pathProperty, nodeIds, onChange]);

  const capitalizedProperty = pathProperty.charAt(0).toUpperCase() + pathProperty.slice(1);

  return (
    <div className="flex items-center space-x-1">
      <select
        value={pathNode}
        onChange={(e) => setPathNode(e.currentTarget.value)}
        className="h-6 rounded bg-neutral-200 px-1 transition"
      >
        {nodeIds.map((id) => (
          <NodeOption key={id} id={id} />
        ))}
      </select>

      <div>/</div>

      <select
        value={capitalizedProperty}
        onChange={(e) => {
          setPathProperty(e.currentTarget.value.toLowerCase());
        }}
        className="h-6 rounded bg-neutral-200 px-1 transition"
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

function NodeOption({ id }: { id: string }) {
  const name = useNodeAttribute(id, "name");

  return (
    <option key={id} value={id} className="text-lg">
      {name}
    </option>
  );
}
