import { Socket } from "socket.io";

import { SocketEvents } from "@wired-xr/engine";

export type TypedSocket = Socket<SocketEvents, SocketEvents>;
