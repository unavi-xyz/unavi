const { createServer } = require("http");
const { Server } = require("socket.io");

const express = require("express");
const app = express();

app.get("/", (req, res) => {
  res.sendFile(path.join(__dirname, "/index.html"));
});

const server = createServer(app);
const io = new Server(server, {
  cors: { origin: "*" },
});

io.on("connection", (socket) => {
  socket.on("players", (room, callback) => {
    const clients = io.sockets.adapter.rooms.get(room);
    const clone = new Set(clients);
    clone.delete(socket.id);
    callback(Array.from(clone));
  });

  socket.on("offer", (target, offer) => {
    // //require the sender to share a room with the target
    // const senderRooms = Array.from(socket.rooms);
    // const targetRooms = new Set(io.sockets.sockets.get(target)?.rooms);
    // targetRooms.delete(target);

    // const intersection = Array.from(targetRooms).filter((value) =>
    //   senderRooms.includes(value)
    // );
    // if (intersection.length === 0) return;

    io.to(target).emit("offer", socket.id, offer);
  });

  socket.on("answer", (target, answer) => {
    io.to(target).emit("answer", socket.id, answer);
  });

  socket.on("iceCandidate", (target, iceCandidate) => {
    io.to(target).emit("iceCandidate", socket.id, iceCandidate);
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

server.listen(8080, "localhost");
