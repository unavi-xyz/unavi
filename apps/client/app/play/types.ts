import { WorldId } from "@/src/utils/parseWorldId";

export type WorldUriId = WorldId | { type: "uri"; value: string };
