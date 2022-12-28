import { FromHostMessage } from "protocol";
import uWS from "uWebSockets.js";

export function send(ws: uWS.WebSocket, message: FromHostMessage) {
  ws.send(JSON.stringify(message));
}
