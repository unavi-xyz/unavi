import { createContext, useEffect, useState } from "react";
import { io, Socket } from "socket.io-client";

const url = "ws://localhost:8080";

interface ISocketContext {
  socket: Socket;
}

const defaultValue: ISocketContext = {
  socket: undefined,
};

export const SocketContext = createContext(defaultValue);

export default function SocketProvider({ children }) {
  const [socket, setSocket] = useState<Socket>();

  useEffect(() => {
    const newSocket = io(url);
    setSocket(newSocket);

    return () => {
      newSocket.disconnect();
      setSocket(undefined);
    };
  }, []);

  return (
    <SocketContext.Provider value={{ socket }}>
      {children}
    </SocketContext.Provider>
  );
}
