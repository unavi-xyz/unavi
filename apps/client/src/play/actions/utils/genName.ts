import { nanoidLowercase } from "@/src/server/nanoid";

export function genName(prefix: string) {
  return `${prefix}_${nanoidLowercase(6)}`;
}
