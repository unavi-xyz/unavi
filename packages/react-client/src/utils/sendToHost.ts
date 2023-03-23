import { ToHostMessage } from "@wired-labs/protocol";

export function sendMessage(ws: WebSocket | null, message: ToHostMessage) {
  if (!ws || ws.readyState !== ws.OPEN) return;

  ws.send(JSON.stringify(message));
}
