export interface Utils<T, JSON> {
  store: Map<string, T>;

  getId: (t: T) => string | undefined;
  create: (json: Partial<JSON> | undefined, id?: string) => { id: string; object: T };
  process: (t: T, id?: string) => { id: string };
  processChanges: () => T[];
  applyJSON: (t: T, json: Partial<JSON>) => void;
  toJSON: (t: T) => JSON;
}
