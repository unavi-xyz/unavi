import {
  AddMesh,
  AddNode,
  EditMesh,
  EditNode,
  RemoveMesh,
  RemoveNode,
} from "@unavi/protocol";
import { PlayerJoined, PlayerLeft } from "@wired-protocol/types";

interface BaseMessage {
  id: number;
  timestamp: number;
  text: string;
}

export interface PlayerMessage extends BaseMessage {
  type: "player";
  playerId: number;
}

export interface SystemMessage extends BaseMessage {
  type: "system";
}

export type ChatMessage = PlayerMessage | SystemMessage;

export type EcsEvent =
  | PlayerJoined
  | PlayerLeft
  | AddNode
  | AddMesh
  | EditNode
  | EditMesh
  | RemoveNode
  | RemoveMesh;
