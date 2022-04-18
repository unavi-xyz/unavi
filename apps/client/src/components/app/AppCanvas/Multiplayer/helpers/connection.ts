import { Socket } from "socket.io-client";

export async function createAnswer(
  connection: RTCPeerConnection,
  socket: Socket,
  id: string
) {
  const answer = await connection.createAnswer();
  connection.setLocalDescription(answer);
  socket.emit("answer", id, answer);
}

export async function createOffer(
  connection: RTCPeerConnection,
  socket: Socket,
  id: string
) {
  const offer = await connection.createOffer();
  connection.setLocalDescription(offer);
  socket.emit("offer", id, offer);
}
