import { createContext, useEffect, useState } from "react";
import { io, Socket } from "socket.io-client";

const url = "ws://localhost:8080";

interface ISocketContext {
  socket: Socket;
  getUserMedia: () => void;
}

const defaultValue: ISocketContext = {
  socket: undefined,
  getUserMedia: undefined,
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

  async function getUserMedia() {
    // const localStream = await navigator.mediaDevices.getUserMedia({
    //   audio: true,
    //   video: false,
    // });
    // localStream.getTracks().forEach((track) => {
    //   localConnection.addTrack(track);
    // });
  }

  return (
    <SocketContext.Provider value={{ socket, getUserMedia }}>
      {children}
    </SocketContext.Provider>
  );
}
