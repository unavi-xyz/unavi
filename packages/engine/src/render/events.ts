import { MessageEvent } from "../types";

export type RenderEvent =
  | MessageEvent<
      "clicked_node",
      { nodeId: string | null; isAvatar: boolean; distance: number | null }
    >
  | MessageEvent<
      "hovered_node",
      { nodeId: string | null; isAvatar: boolean; distance: number | null }
    >;
