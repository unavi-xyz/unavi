import { EventDispatcher } from "property-graph";

import { MessageEvent } from "../../types";

type AttributeEvent = MessageEvent<"create", { id: string }>;

export abstract class Attribute<T, JSON> extends EventDispatcher<AttributeEvent> {
  abstract store: Map<string, T>;

  constructor() {
    super();
  }

  abstract getId(t: T): string | undefined;
  abstract create(json: Partial<JSON> | undefined, id?: string): { id: string; object: T };
  abstract process(t: T, id?: string): { id: string };
  abstract processChanges(): T[];
  abstract applyJSON(t: T, json: Partial<JSON>): void;
  abstract toJSON(t: T): JSON;

  emitCreate(id: string) {
    this.dispatchEvent({ type: "create", data: { id } });
  }
}
