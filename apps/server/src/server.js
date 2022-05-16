const { createServer } = require("http");
const { Server } = require("socket.io");

const httpServer = createServer();
const io = new Server(httpServer, {
  cors: { origin: "*" },
});

io.on("connection", (socket) => {
  console.log("✅ user connected:", socket.id);

  //join a room
  socket.on("join", (room) => {
    socket.join(room);
  });

  //leave all rooms on disconnect
  socket.on("disconnecting", () => {
    console.log("❌ user disconnected:", socket.id);
    const rooms = Array.from(socket.rooms);
    //the first room is just your own id
    rooms.shift();
    io.to(rooms).emit("leave", socket.id);
  });

  //get all players in a room
  socket.on("players", (room, callback) => {
    const clients = io.sockets.adapter.rooms.get(room);
    const clone = new Set(clients);
    clone.delete(socket.id);
    callback(Array.from(clone));
  });

  //send a webrtc offer to a player
  socket.on("offer", (target, offer) => {
    io.to(target).emit("offer", socket.id, offer);
  });

  //accept a recieved webrtc offer
  socket.on("answer", (target, answer) => {
    io.to(target).emit("answer", socket.id, answer);
  });

  //send a webrtc ice candidate to a player
  socket.on("iceCandidate", (target, iceCandidate) => {
    io.to(target).emit("iceCandidate", socket.id, iceCandidate);
  });
});

const port = process.env.PORT || 8080;

httpServer.listen(port, () => {
  console.log(`server is running on port ${port}`);
});
