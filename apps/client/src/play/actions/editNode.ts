import { useClientStore } from "@unavi/engine";
import { EditNode } from "@unavi/protocol";

export function editNode(data: EditNode["data"]) {
  useClientStore.getState().mirrorEvent({
    data,
    id: "xyz.unavi.editor.edit.node",
    target: "client",
  });
}
