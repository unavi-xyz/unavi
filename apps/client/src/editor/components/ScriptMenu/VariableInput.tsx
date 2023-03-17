import * as Select from "@radix-ui/react-select";
import { ValueType, Variable } from "engine";
import { useEffect, useState } from "react";
import { IoIosArrowDown } from "react-icons/io";

import { useEditorStore } from "../../store";
import { useVariableAttribute } from "./hooks/useVariableAttribute";
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
  const [variableId, setVariableId] = useState<number>();

  const isPlaying = useEditorStore((state) => state.isPlaying);
  const variables = useEditorStore((state) => state.variables);
  const variable = variableId !== undefined ? variables[variableId] ?? null : null;

  const variableName = useVariableAttribute(variable, "name") ?? "";
  const variableType = useVariableAttribute(variable, "type") ?? ValueType.string;

  // Load initial variable
  useEffect(() => {
    const { engine } = useEditorStore.getState();
    if (!engine) return;

    const param = data["variable"];

    // If no variables, create one
    if (variables.length === 0) {
      const newVariable = engine.scene.extensions.behavior.createVariable();
      newVariable.setName(`Variable ${variables.length}`);
      useEditorStore.setState({ variables: [...variables, newVariable] });
    }

    // If variable is set, use it
    if (param && flowIsVariableJSON(param)) {
      const variable = variables[param.variableId];
      if (variable !== undefined) {
        setVariableId(param.variableId);
        return;
      }
    }

    // Set default variable
    setVariableId(0);

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data]);

  // Update node data
  useEffect(() => {
    if (variableId === undefined) return;
    onChange("variable", { variableId });
  }, [onChange, variableId, variables]);

  return (
    <div className="relative">
      <div className="flex h-7 items-center space-x-1">
        <div>Variable</div>

        <Select.Root
          value={String(variableId)}
          disabled={isPlaying}
          onValueChange={(value) => {
            const { engine } = useEditorStore.getState();
            if (!engine) return;

            const isNewVariable = Number(value) === variables.length;

            if (isNewVariable) {
              // Create new variable
              const newVariable = engine.scene.extensions.behavior.createVariable();
              newVariable.setName(`Variable ${variables.length}`);
              useEditorStore.setState({ variables: [...variables, newVariable] });
            }

            // Set new variable id
            const newId = isNewVariable ? variables.length : Number(value);
            setVariableId(newId);
          }}
        >
          <div>
            <div className="flex h-6 w-36 rounded bg-neutral-200">
              <input
                value={variableName}
                disabled={isPlaying}
                onChange={(e) => {
                  if (!variable) return;
                  variable.setName(e.currentTarget.value);
                }}
                // eslint-disable-next-line tailwindcss/no-custom-classname
                className={`nodrag h-full w-32 rounded-l bg-neutral-200 px-2 focus:outline-none ${
                  isPlaying ? "" : "hover:bg-neutral-300/80 focus:bg-neutral-300/80"
                }`}
              />

              <Select.Trigger
                className={`flex h-full w-full items-center justify-center rounded-r ${
                  isPlaying ? "" : "hover:bg-neutral-300/80 focus:outline-none"
                }`}
              >
                <IoIosArrowDown />
              </Select.Trigger>
            </div>

            <Select.Content className="w-36">
              <Select.Viewport className="rounded-md bg-neutral-200 shadow-lg">
                {variables.map((variable, i) => (
                  <VariableItem key={i} value={String(i)} variable={variable} />
                ))}

                <Select.Item
                  value={String(variables.length)}
                  className="cursor-default px-4 focus:bg-blue-500 focus:text-white focus:outline-none"
                >
                  <Select.ItemText className="text-lg">+ New Variable</Select.ItemText>
                </Select.Item>
              </Select.Viewport>
            </Select.Content>
          </div>
        </Select.Root>

        <div>as</div>

        <select
          value={variableType}
          disabled={isPlaying}
          onChange={(e) => {
            if (!variable) return;
            variable.type = e.currentTarget.value;
          }}
          // eslint-disable-next-line tailwindcss/no-custom-classname
          className={`nodrag h-6 rounded bg-neutral-200 px-1 ${
            isPlaying ? "" : "hover:bg-neutral-300/80 focus:bg-neutral-300/80 focus:outline-none"
          }`}
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

function VariableItem({ value, variable }: { value: string; variable: Variable }) {
  const name = useVariableAttribute(variable, "name") ?? "";

  return (
    <Select.Item
      value={value}
      className="cursor-default px-4 focus:bg-blue-500 focus:text-white focus:outline-none"
    >
      <Select.ItemText className="text-lg">{name}</Select.ItemText>
    </Select.Item>
  );
}
