import { SpaceId } from "@/src/utils/parseSpaceId";

export type SpaceUriId = SpaceId | { type: "uri"; value: string };
