const { createServer } = require("http");
const { Server } = require("socket.io");

const httpServer = createServer();
const io = new Server(httpServer, {
  cors: { origin: "*" },
});

io.on("connection", (socket) => {
  socket.on("players", (room, callback) => {
    const clients = io.sockets.adapter.rooms.get(room);
    const clone = new Set(clients);
    clone.delete(socket.id);
    callback(Array.from(clone));
  });

  socket.on("offer", (player, offer) => {
    io.to(player).emit("offer", socket.id, offer);
  });

  socket.on("answer", (player, answer) => {
    io.to(player).emit("answer", socket.id, answer);
  });

  socket.on("iceCandidate", (player, iceCandidate) => {
    io.to(player).emit("iceCandidate", socket.id, iceCandidate);
  });

  socket.on("join", (room) => {
    socket.join(room);
  });

  socket.on("disconnecting", () => {
    const rooms = Array.from(socket.rooms);
    rooms.shift();
    io.to(rooms).emit("leave", socket.id);
  });
});

httpServer.listen(8080);
