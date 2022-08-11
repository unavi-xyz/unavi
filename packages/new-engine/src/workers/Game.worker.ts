import { GameWorker } from "../game/GameWorker";
import { ToGameMessage } from "../types";

const gameWorker = new GameWorker();

onmessage = (event: MessageEvent<ToGameMessage>) => {
  const { subject, data } = event.data;

  switch (subject) {
    case "init_player":
      gameWorker.initPlayer();
      break;
  }
};
