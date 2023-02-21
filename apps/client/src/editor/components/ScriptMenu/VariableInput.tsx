import { ValueType } from "engine";
import { nanoid } from "nanoid";
import { useEffect, useState } from "react";

import { useEditorStore } from "../../store";
import { FlowNodeData, FlowNodeParamter } from "./types";
import { flowIsVariableJSON } from "./utils/filters";

const VALUE_TYPES = [
  ValueType.string,
  ValueType.float,
  ValueType.boolean,
  ValueType.vec2,
  ValueType.vec3,
  ValueType.quat,
];

interface Props {
  data: FlowNodeData;
  onChange: (key: string, value: FlowNodeParamter) => void;
}

export default function VariableInput({ data, onChange }: Props) {
  const [valueType, setValueType] = useState<string>(ValueType.string);
  const [variableId, setVariableId] = useState<number>();

  const variables = useEditorStore((state) => state.variables);

  // Load variable id
  useEffect(() => {
    const param = data["variable"];
    if (!param || !flowIsVariableJSON(param)) return;

    setVariableId(param.variableId);
  }, [data]);

  // Update variable type
  useEffect(() => {
    const param = data["variable"];
    if (!param || !flowIsVariableJSON(param)) return;

    const variable = variables[param.variableId];
    if (!variable) return;

    setValueType(variable.type);
  }, [variables, data]);

  // Update node data
  useEffect(() => {
    if (variableId === undefined) return;

    const param = data["variable"];

    if (param && flowIsVariableJSON(param)) {
      const didChangeId = param.variableId !== variableId;

      // Only change variable type if id didn't change
      if (!didChangeId) {
        const variable = variables[variableId];
        if (variable) variable.type = valueType;
      }
    }

    onChange("variable", { variableId });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [onChange, variableId, valueType, variables]);

  return (
    <div>
      <div className="flex h-7 items-center space-x-1">
        <div>Variable</div>

        <select
          value={variableId}
          onChange={(e) => {
            const { engine } = useEditorStore.getState();
            if (!engine) return;

            const isNewVariable = Number(e.currentTarget.value) === variables.length;

            if (isNewVariable) {
              // Create new variable
              const newVariable = engine.scene.extensions.behavior.createVariable();
              newVariable.type = valueType;
              newVariable.setName(nanoid(8));
              useEditorStore.setState({ variables: [...variables, newVariable] });
            }

            // Set new variable id
            const value = isNewVariable ? variables.length : Number(e.currentTarget.value);
            setVariableId(value);
          }}
          // eslint-disable-next-line tailwindcss/no-custom-classname
          className="nodrag h-6 rounded bg-neutral-200 px-1 hover:bg-neutral-300/80 focus:bg-neutral-300/80"
        >
          {variables.map((variable, i) => (
            <option key={i} value={i} className="text-lg">
              {variable.getName()}
            </option>
          ))}

          <option value={variables.length} className="text-lg">
            + New Variable
          </option>
        </select>

        <div>as</div>

        <select
          value={valueType}
          onChange={(e) => setValueType(e.currentTarget.value)}
          // eslint-disable-next-line tailwindcss/no-custom-classname
          className="nodrag h-6 rounded bg-neutral-200 px-1 hover:bg-neutral-300/80 focus:bg-neutral-300/80"
        >
          {VALUE_TYPES.map((type) => (
            <option key={type} value={type} className="text-lg">
              {type}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
}
