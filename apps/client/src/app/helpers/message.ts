import { nanoid } from "nanoid";
import { Message } from "./types";

export function createMessage(text: string) {
  const id = nanoid();
  const time = Date.now();
  const message: Message = { id, time, text, username: undefined };
  return message;
}
