export type IChatMessage = {
  id: string;
  playerId: string;
  message: string;
  timestamp: number;
};

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
    >
  | GenericWebSocketMessage<"message", string>
  | GenericWebSocketMessage<"falling_state", boolean>;

export type FromHostMessage =
  | GenericWebSocketMessage<"player_joined", string>
  | GenericWebSocketMessage<"player_left", string>
  | GenericWebSocketMessage<
      "player_location",
      {
        playerId: string;
        location: [number, number, number, number, number, number, number];
      }
    >
  | GenericWebSocketMessage<"player_message", IChatMessage>
  | GenericWebSocketMessage<
      "player_falling_state",
      {
        playerId: string;
        isFalling: boolean;
      }
    >;
