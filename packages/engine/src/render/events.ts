import { MessageEvent } from "../types";

export type RenderEvent = MessageEvent<"clicked_node", { nodeId: string | null }>;
