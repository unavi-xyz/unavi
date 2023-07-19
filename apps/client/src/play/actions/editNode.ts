import { EditNode } from "@unavi/protocol";
import { useClientStore } from "@unavi/react-client";

export function editNode(data: EditNode["data"]) {
  useClientStore.getState().mirrorEvent({
    data,
    id: "xyz.unavi.editor.edit.node",
    target: "client",
  });
}
