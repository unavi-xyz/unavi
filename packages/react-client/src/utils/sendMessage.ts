import { RequestMessage } from "@wired-protocol/types";

export function sendMessage(ws: WebSocket | null, message: RequestMessage) {
  if (!ws || ws.readyState !== ws.OPEN) return;

  ws.send(JSON.stringify(message));
}
