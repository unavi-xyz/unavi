import { useClientStore } from "@unavi/engine";
import { EditNode } from "@unavi/protocol";

export function editNode(data: Parameters<typeof EditNode.create>[0]) {
  const msg = EditNode.create(data);
  useClientStore.getState().mirrorEvent(EditNode.toBinary(msg));
}
