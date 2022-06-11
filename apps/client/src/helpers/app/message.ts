import { nanoid } from "nanoid";

import { IChatMessage } from "./types";

export function createMessage(text: string, username: string) {
  const id = nanoid();
  const time = Date.now();
  const message: IChatMessage = {
    id,
    timestamp: time,
    message: text,
    username,
  };
  return message;
}
