import { FromHostMessage } from "@wired-labs/engine";
import uWS from "uWebSockets.js";

export function send(ws: uWS.WebSocket, message: FromHostMessage) {
  ws.send(JSON.stringify(message));
}
