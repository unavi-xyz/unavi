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
  | GenericWebSocketMessage<"leave", null>
  | GenericWebSocketMessage<
      "location",
      [number, number, number, number, number, number, number]
    >;

export type FromHostMessage =
  | GenericWebSocketMessage<"player_joined", string>
  | GenericWebSocketMessage<"player_left", string>
  | GenericWebSocketMessage<
      "player_location",
      {
        playerId: string;
        location: [number, number, number, number, number, number, number];
      }
    >;
