import { EditorEvent } from "@unavi/protocol";
import { EventWriter } from "thyseus";
import { applyUpdateV2 } from "yjs";

import { TxOrigin } from "../classes/YjsProvider";
import { useClientStore } from "../clientStore";
import { PlayerJoin, PlayerLeave } from "../events";
import { ydoc } from "../yjs";

/**
 * Converts networked events into ECS events
 */
export function sendEvents(
  playerJoin: EventWriter<PlayerJoin>,
  playerLeave: EventWriter<PlayerLeave>
) {
  const ecsIncoming = useClientStore.getState().ecsIncoming;
  if (ecsIncoming.length === 0) return;

  let msgKind: string | undefined;
  let numProcessed = 0;

  for (const msg of ecsIncoming) {
    let kind: string | undefined = msg.response.oneofKind;
    let editor: EditorEvent | undefined;

    if (msg.response.oneofKind === "event") {
      editor = EditorEvent.fromBinary(msg.response.event.data);
      kind = `event.${editor.event.oneofKind}`;
    }

    if (!msgKind) {
      msgKind = kind;
    } else if (msgKind !== kind) {
      // Only process one kind of message per frame
      // This is to simplify message handling
      break;
    }

    numProcessed++;

    switch (msg.response.oneofKind) {
      case "playerJoined": {
        const e = new PlayerJoin();
        e.playerId = msg.response.playerJoined.playerId;
        playerJoin.create(e);
        break;
      }

      case "playerLeft": {
        const e = new PlayerLeave();
        e.playerId = msg.response.playerLeft.playerId;
        playerLeave.create(e);
        break;
      }

      case "event": {
        if (!editor) {
          console.error("No editor event");
          break;
        }

        switch (editor.event.oneofKind) {
          case "syncScene": {
            applyUpdateV2(ydoc, editor.event.syncScene.data, TxOrigin.Remote);
          }
        }
      }
    }
  }

  for (let i = 0; i < numProcessed; i++) {
    ecsIncoming.shift();
  }
}
