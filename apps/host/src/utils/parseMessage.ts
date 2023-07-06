import { WebSocketMessageSchema } from "@wired-protocol/types";

const textDecoder = new TextDecoder();

export function parseMessage(buffer: ArrayBuffer) {
  try {
    const text = textDecoder.decode(buffer);
    const json = JSON.parse(text);
    const parsed = WebSocketMessageSchema.parse(json);
    return parsed;
  } catch (error) {
    console.warn(error);
    return;
  }
}
