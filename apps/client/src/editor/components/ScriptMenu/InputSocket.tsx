import { InputSocketSpecJSON } from "@unavi/behave-graph-core";
import { ConstantValue, ValueType } from "@unavi/gltf-extensions";
import { FaCaretRight } from "react-icons/fa";
import { Connection, Handle, Position, useReactFlow } from "reactflow";

import { useEditor } from "../Editor";
import AutoSizeInput from "./AutoSizeInput";
import JsonPathInput from "./JsonPathInput";
import { useScript } from "./Script";
import { FlowNodeParamter } from "./types";
import { valueColorsMap } from "./utils/colors";
import { isValidConnection } from "./utils/isValidConnection";

export interface InputSocketProps extends InputSocketSpecJSON {
  connected: boolean;
  value?: ConstantValue;
  pathType?: string;
  onChange: (key: string, value: FlowNodeParamter) => void;
}

export default function InputSocket({
  connected,
  value,
  onChange,
  name,
  valueType,
  pathType,
}: InputSocketProps) {
  const { mode } = useEditor();
  const { variables } = useScript();
  const instance = useReactFlow();

  const isFlowSocket = valueType === "flow";
  const isJsonPath = name === "jsonPath";
  const showName = isFlowSocket === false || name !== "flow";
  const displayName = isJsonPath ? "Path" : name;
  const inputType =
    valueType === ValueType.string
      ? "text"
      : valueType === ValueType.number
      ? "number"
      : valueType === ValueType.float
      ? "number"
      : valueType === ValueType.integer
      ? "number"
      : valueType === ValueType.boolean
      ? "checkbox"
      : "";

  return (
    <div className="flex h-7 grow items-center">
      {isFlowSocket && <FaCaretRight />}
      {showName && <div className="mr-2 capitalize">{displayName}</div>}

      {isFlowSocket === false && connected === false ? (
        // eslint-disable-next-line tailwindcss/no-custom-classname
        <div className="nodrag">
          {isJsonPath ? (
            <JsonPathInput
              pathType={pathType}
              value={String(value?.value ?? "")}
              onChange={onChange}
            />
          ) : (
            <AutoSizeInput
              type={inputType}
              value={String(value?.value ?? "")}
              disabled={mode === "play"}
              onChange={(e) => onChange(name, { value: e.currentTarget.value })}
              className={`h-6 rounded bg-neutral-200 px-2 ${
                mode === "play"
                  ? ""
                  : "hover:bg-neutral-300/80 focus:bg-neutral-300/80 focus:outline-none"
              }`}
            />
          )}
        </div>
      ) : null}

      {!isJsonPath && (
        <Handle
          id={name}
          type="target"
          position={Position.Left}
          isValidConnection={(connection: Connection) =>
            isValidConnection(connection, instance, variables)
          }
          style={{
            borderColor: valueColorsMap[valueType],
            backgroundColor: connected ? "#262626" : "#ffffff",
          }}
        />
      )}
    </div>
  );
}
