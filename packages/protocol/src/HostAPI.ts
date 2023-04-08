/**
 * Static class for generating API paths.
 */
export class HostAPI {
  static playerCount = (uri: string) => `player-count/${uri}` as const;
}
