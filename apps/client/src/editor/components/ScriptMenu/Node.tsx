import { NodeCategory, NodeSpecJSON } from "@wired-labs/behave-graph-core";
import { ConstantValue } from "engine";
import { NodeProps, useEdges } from "reactflow";

import { useEditorStore } from "../../store";
import { useChangeNodeData } from "./hooks/useChangeNodeData";
import InputSocket from "./InputSocket";
import NodeContainer from "./NodeContainer";
import OutputSocket from "./OutputSocket";
import { FlowNodeData } from "./types";
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
  const edges = useEdges();
  const handleChange = useChangeNodeData(id);
  const pairs = getPairs(spec.inputs, spec.outputs);

  const variables = useEditorStore((state) => state.variables);

  const isVariable = spec.type === "variable/get" || spec.type === "variable/set";
  const category = isVariable ? NodeCategory.Variable : spec.category;

  return (
    <NodeContainer id={id} title={spec.label} category={category} selected={selected}>
      {pairs.map(([input, output], i) => {
        const defaultValue: ConstantValue | undefined = input?.defaultValue
          ? { value: input.defaultValue as any }
          : undefined;

        const inputValue = input ? data[input.name] : undefined;

        const value = input
          ? flowIsConstantJSON(inputValue)
            ? inputValue
            : defaultValue
          : undefined;

        let variableType: string | undefined;

        if (spec.type === "variable/get" || (spec.type === "variable/set" && i == 1)) {
          const variableJson = data["variable"];
          if (flowIsVariableJSON(variableJson)) {
            const variableId = variableJson.variableId;
            const variable = variables[variableId];
            if (variable) {
              variableType = variable.type;
            }
          }
        }

        const inputPathType = spec.type.includes("scene/set")
          ? pairs[i + 1]?.[0]?.valueType
          : output?.valueType;

        return (
          <div key={i} className="relative flex flex-row justify-between gap-8 px-2">
            {spec.type === "variable/get" && <VariableInput data={data} onChange={handleChange} />}

            {input && (
              <InputSocket
                name={input.name}
                valueType={variableType ?? input.valueType}
                pathType={inputPathType}
                value={value}
                onChange={handleChange}
                connected={isHandleConnected(edges, id, input.name, "target")}
              />
            )}

            {output && (
              <OutputSocket
                name={output.name}
                valueType={variableType ?? output.valueType}
                connected={isHandleConnected(edges, id, output.name, "source")}
              />
            )}

            {spec.type === "variable/set" && i == 1 && (
              <VariableInput data={data} onChange={handleChange} />
            )}
          </div>
        );
      })}
    </NodeContainer>
  );
}
