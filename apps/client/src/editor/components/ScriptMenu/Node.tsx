import { NodeSpecJSON } from "behave-graph";
import { NodeProps, useEdges } from "reactflow";

import { useChangeNodeData } from "./hooks/useChangeNodeData";
import InputSocket from "./InputSocket";
import NodeContainer from "./NodeContainer";
import OutputSocket from "./OutputSocket";
import { isHandleConnected } from "./utils/isHandleConnected";

interface Props extends NodeProps {
  spec: NodeSpecJSON;
}

const getPairs = <T, U>(arr1: T[], arr2: U[]) => {
  const max = Math.max(arr1.length, arr2.length);
  const pairs = [];
  for (let i = 0; i < max; i++) {
    const pair: [T | undefined, U | undefined] = [arr1[i], arr2[i]];
    pairs.push(pair);
  }
  return pairs;
};

export default function Node({ id, data, spec, selected }: Props) {
  const edges = useEdges();
  const handleChange = useChangeNodeData(id);
  const pairs = getPairs(spec.inputs, spec.outputs);

  return (
    <NodeContainer title={spec.label} category={spec.category} selected={selected}>
      {pairs.map(([input, output], i) => (
        <div key={i} className="relative flex flex-row justify-between gap-8 px-2">
          {input && (
            <InputSocket
              {...input}
              value={data[input.name] ?? input.defaultValue}
              onChange={handleChange}
              connected={isHandleConnected(edges, id, input.name, "target")}
            />
          )}

          {output && (
            <OutputSocket
              {...output}
              connected={isHandleConnected(edges, id, output.name, "source")}
            />
          )}
        </div>
      ))}
    </NodeContainer>
  );
}
