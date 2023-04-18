import { OutputSocketSpecJSON } from "@unavi/behave-graph-core";
import { FaCaretRight } from "react-icons/fa";
import { Connection, Handle, Position, useReactFlow } from "reactflow";

import { useScript } from "./Script";
import { valueColorsMap } from "./utils/colors";
import { isValidConnection } from "./utils/isValidConnection";

export interface OutputSocketProps extends OutputSocketSpecJSON {
  connected: boolean;
}

export default function OutputSocket({ connected, valueType, name }: OutputSocketProps) {
  const { variables } = useScript();
  const instance = useReactFlow();
  const isFlowSocket = valueType === "flow";
  const showName = isFlowSocket === false || name !== "flow";

  return (
    <div className="flex h-7 grow items-center justify-end">
      {showName && <div className="capitalize">{name}</div>}
      {isFlowSocket && <FaCaretRight className="pl-2" />}

      <Handle
        id={name}
        type="source"
        position={Position.Right}
        isValidConnection={(connection: Connection) =>
          isValidConnection(connection, instance, variables)
        }
        style={{
          borderColor: valueColorsMap[valueType],
          backgroundColor: connected ? "#262626" : "#ffffff",
        }}
      />
    </div>
  );
}
