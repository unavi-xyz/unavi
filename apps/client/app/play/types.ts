import { SpaceId } from "@/src/utils/parseSpaceId";

export type WorldUriId = SpaceId | { type: "uri"; value: string };
