/**
 * Static class for generating API paths.
 */
export class HostAPI {
  static space = (id: number) =>
    ({
      playerCount: `spaces/${id}/player-count`,
    } as const);
}
