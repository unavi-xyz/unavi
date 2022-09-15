import { SocketEvents } from "@wired-labs/engine";
import { Socket } from "socket.io";

export type TypedSocket = Socket<SocketEvents, SocketEvents>;
