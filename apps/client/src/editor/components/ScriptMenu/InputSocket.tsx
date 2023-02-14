import { InputSocketSpecJSON } from "@behave-graph/core";
import { FaCaretRight } from "react-icons/fa";
import { Connection, Handle, Position, useReactFlow } from "reactflow";

import AutoSizeInput from "./AutoSizeInput";
import JsonPathInput from "./JsonPathInput";
import { valueColorsMap } from "./utils/colors";
import { isValidConnection } from "./utils/isValidConnection";

export interface InputSocketProps extends InputSocketSpecJSON {
  connected: boolean;
  value: any | undefined;
  onChange: (key: string, value: any) => void;
}

export default function InputSocket({
  connected,
  value,
  onChange,
  name,
  valueType,
  defaultValue,
}: InputSocketProps) {
  const instance = useReactFlow();

  const isFlowSocket = valueType === "flow";
  const showName = isFlowSocket === false || name !== "flow";
  const displayName = name === "jsonPath" ? "Path" : name;
  const inputType =
    valueType === "string"
      ? "text"
      : valueType === "number"
      ? "number"
      : valueType === "float"
      ? "number"
      : valueType === "integer"
      ? "number"
      : valueType === "boolean"
      ? "checkbox"
      : "";

  return (
    <div className="flex h-7 grow items-center">
      {isFlowSocket && <FaCaretRight />}
      {showName && <div className="mr-2 capitalize">{displayName}</div>}

      {isFlowSocket === false && connected === false ? (
        // eslint-disable-next-line tailwindcss/no-custom-classname
        <div className="nodrag">
          {name === "jsonPath" ? (
            <JsonPathInput onChange={onChange} value={String(value)} />
          ) : (
            <AutoSizeInput
              type={inputType}
              className="h-6 rounded bg-neutral-200 px-2 transition hover:bg-neutral-300/80 focus:bg-neutral-300/80"
              value={String(value) ?? defaultValue ?? ""}
              onChange={(e) => onChange(name, e.currentTarget.value)}
            />
          )}
        </div>
      ) : null}

      <Handle
        id={name}
        type="target"
        position={Position.Left}
        isValidConnection={(connection: Connection) => isValidConnection(connection, instance)}
        style={{
          borderColor: valueColorsMap[valueType],
          backgroundColor: connected ? "#262626" : "#ffffff",
        }}
      />
    </div>
  );
}
