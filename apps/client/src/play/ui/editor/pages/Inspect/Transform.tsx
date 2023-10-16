import { SyncedNode } from "@unavi/engine";

import { DeepReadonly } from "@/src/play/utils/types";

import Rotation from "./Rotation";
import Scale from "./Scale";
import Translation from "./Translation";

interface Props {
  node: DeepReadonly<SyncedNode>;
}

export default function Transform({ node }: Props) {
  return (
    <div className="space-y-1">
      <Translation node={node} />
      <Rotation node={node} />
      <Scale node={node} />
    </div>
  );
}
