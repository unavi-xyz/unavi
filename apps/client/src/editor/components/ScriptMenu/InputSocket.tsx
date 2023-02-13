import { InputSocketSpecJSON } from "@behave-graph/core";
import { FaCaretRight } from "react-icons/fa";
import { Connection, Handle, Position, useReactFlow } from "reactflow";

import AutoSizeInput from "./AutoSizeInput";
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

  const type =
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
    <div className="flex h-7 grow items-center justify-start">
      {isFlowSocket && <FaCaretRight />}
      {showName && <div className="mr-2 capitalize">{name}</div>}

      {isFlowSocket === false && connected === false && (
        <AutoSizeInput
          type={type}
          // eslint-disable-next-line tailwindcss/no-custom-classname
          className="nodrag rounded bg-neutral-200 px-2 py-0.5 transition hover:bg-neutral-300/80 focus:bg-neutral-300/80"
          value={String(value) ?? defaultValue ?? ""}
          onChange={(e) => onChange(name, e.currentTarget.value)}
        />
      )}

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
