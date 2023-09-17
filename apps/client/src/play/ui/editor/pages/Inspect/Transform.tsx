import { SyncedNode } from "@unavi/engine";

import { DeepReadonly } from "@/src/play/utils/types";

import Translation from "./Translation";

interface Props {
  node: DeepReadonly<SyncedNode>;
}

export default function Transform({ node }: Props) {
  return (
    <div className="space-y-1">
      <Translation node={node} />
    </div>
  );
}
