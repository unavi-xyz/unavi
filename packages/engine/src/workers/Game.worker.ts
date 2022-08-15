import { GameWorker } from "../game/GameWorker";
import { FromGameMessage, ToGameMessage } from "../types";

const gameWorker = new GameWorker();

onmessage = (event: MessageEvent<ToGameMessage>) => {
  const { subject, data } = event.data;

  switch (subject) {
    case "init_player":
      gameWorker.initPlayer();
      break;
    case "jumping":
      gameWorker.setJumping(data);
      break;
  }
};

const message: FromGameMessage = {
  subject: "ready",
  data: null,
};
postMessage(message);

console.log("👩‍❤️‍👩", typeof window);
