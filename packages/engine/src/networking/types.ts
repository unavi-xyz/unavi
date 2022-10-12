export type InternalChatMessage = {
  id: string;
  playerId: string;
  username: string;
  message: string;
  timestamp: number;
};

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
  | GenericWebSocketMessage<"falling_state", boolean>
  | GenericWebSocketMessage<"set_name", string>;

export type FromHostMessage =
  | GenericWebSocketMessage<
      "join_successful",
      {
        playerId: string;
      }
    >
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
    >
  | GenericWebSocketMessage<"player_name", { playerId: string; name: string }>;
