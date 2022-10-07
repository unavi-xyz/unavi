type GenericWebSocketMessage<S extends string, D> = {
  subject: S;
  data: D;
};

export type ToHostMessage =
  | GenericWebSocketMessage<
      "join",
      {
        spaceId: string;
      }
    >
  | GenericWebSocketMessage<"leave", null>;

export type FromHostMessage =
  | GenericWebSocketMessage<
      "player_joined",
      {
        playerId: string;
      }
    >
  | GenericWebSocketMessage<
      "player_left",
      {
        playerId: string;
      }
    >;
