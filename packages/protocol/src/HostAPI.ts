/**
 * Static class for generating API paths.
 */
export class HostAPI {
  static space = (uri: string) =>
    ({
      playerCount: `player-count/${uri}`,
    } as const);
}
