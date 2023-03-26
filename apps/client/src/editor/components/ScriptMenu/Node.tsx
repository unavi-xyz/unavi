import { NodeCategory, NodeSpecJSON } from "@wired-labs/behave-graph-core";
import { ConstantValue } from "@wired-labs/gltf-extensions";
import { NodeProps, useEdges } from "reactflow";

import { useEditorStore } from "../../../../app/editor/[id]/store";
import { useChangeNodeData } from "./hooks/useChangeNodeData";
import { useVariableAttribute } from "./hooks/useVariableAttribute";
import InputSocket from "./InputSocket";
import NodeContainer from "./NodeContainer";
import OutputSocket from "./OutputSocket";
import { FlowNodeData, FlowNodeParamter } from "./types";
import { flowIsConstantJSON, flowIsVariableJSON } from "./utils/filters";
import { isHandleConnected } from "./utils/isHandleConnected";
import VariableInput from "./VariableInput";

interface Props extends NodeProps {
  data: FlowNodeData;
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
  const handleChange = useChangeNodeData(id);
  const pairs = getPairs(spec.inputs, spec.outputs);

  const isVariable = spec.type === "variable/get" || spec.type === "variable/set";
  const category = isVariable ? NodeCategory.Variable : spec.category;

  return (
    <NodeContainer id={id} title={spec.label} category={category} selected={selected}>
      {pairs.map(([input, output], i) => (
        <SocketPair
          key={i}
          i={i}
          nodeId={id}
          nextPair={pairs[i + 1]}
          input={input}
          output={output}
          spec={spec}
          data={data}
          onChange={handleChange}
        />
      ))}
    </NodeContainer>
  );
}

interface SocketPairProps {
  i: number;
  nextPair?: [NodeSpecJSON["inputs"][0] | undefined, NodeSpecJSON["outputs"][0] | undefined];
  input?: NodeSpecJSON["inputs"][0];
  output?: NodeSpecJSON["outputs"][0];
  nodeId: string;
  spec: NodeSpecJSON;
  data: FlowNodeData;
  onChange: (key: string, value: FlowNodeParamter) => void;
}

function SocketPair({ i, nextPair, input, output, nodeId, spec, data, onChange }: SocketPairProps) {
  const variables = useEditorStore((state) => state.variables);
  const edges = useEdges();

  const defaultValue: ConstantValue | undefined = input?.defaultValue
    ? { value: input.defaultValue as any }
    : undefined;
  const inputValue = input ? data[input.name] : undefined;
  const value = input ? (flowIsConstantJSON(inputValue) ? inputValue : defaultValue) : undefined;

  const variableJson = data["variable"];
  const variableId = flowIsVariableJSON(variableJson) ? variableJson.variableId : undefined;
  const variable = variableId !== undefined ? variables[variableId] ?? null : null;

  const isVariableRow =
    (spec.type === "variable/get" && i === 0) || (spec.type === "variable/set" && i === 1);
  const variableType = useVariableAttribute(isVariableRow ? variable : null, "type");

  const inputPathType = spec.type.includes("scene/set")
    ? nextPair?.[0]?.valueType
    : output?.valueType;

  return (
    <div className="relative flex flex-row justify-between gap-8 px-2">
      {spec.type === "variable/get" && <VariableInput data={data} onChange={onChange} />}

      {input && (
        <InputSocket
          name={input.name}
          valueType={variableType ?? input.valueType}
          pathType={inputPathType}
          value={value}
          onChange={onChange}
          connected={isHandleConnected(edges, nodeId, input.name, "target")}
        />
      )}

      {output && (
        <OutputSocket
          name={output.name}
          valueType={variableType ?? output.valueType}
          connected={isHandleConnected(edges, nodeId, output.name, "source")}
        />
      )}

      {spec.type === "variable/set" && i == 1 && <VariableInput data={data} onChange={onChange} />}
    </div>
  );
}
