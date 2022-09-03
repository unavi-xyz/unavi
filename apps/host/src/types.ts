import { Socket } from "socket.io";

import { SocketEvents } from "@wired-labs/engine";

export type TypedSocket = Socket<SocketEvents, SocketEvents>;
