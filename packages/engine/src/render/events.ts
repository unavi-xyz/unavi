import { MessageEvent } from "../types";

export type RenderEvent = MessageEvent<"clicked_node", { id: string | null }>;
