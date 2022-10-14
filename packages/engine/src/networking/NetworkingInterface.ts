import {
  GetPublicationDocument,
  GetPublicationQuery,
  GetPublicationQueryVariables,
  Publication,
} from "@wired-labs/lens";
import { BehaviorSubject } from "rxjs";
import { createClient } from "urql";

import { MainScene } from "../main/MainScene";
import { RenderThread } from "../render/RenderThread";
import { Entity, GLTFMesh } from "../scene";
import { LENS_API } from "./constants";
import { FromHostMessage, InternalChatMessage, ToHostMessage } from "./types";

const DEFAULT_HOST = "wss://host.thewired.space";
const PUBLISH_HZ = 20; // X times per second

/*
 * Acts as an interface for all networking functionality.
 * Handles joining spaces, sending and receiving messages, etc.
 */
export class NetworkingInterface {
  #scene: MainScene;
  #renderThread: RenderThread;

  #ws: WebSocket | null = null;
  #lensClient = createClient({ url: LENS_API });
  #spaceEntityId: string | null = null;
  #broadcastInterval: NodeJS.Timeout | null = null;
  #hostServer: string | null = null;
  #reconnectCount = 0;

  #playerPosition: Int32Array | null = null;
  #playerRotation: Int32Array | null = null;

  #playerNames = new Map<string, string>();
  #playerHandles = new Map<string, string>();

  #myName: string | null = null;
  #myAvatar: string | null = null;
  #myHandle: string | null = null;

  playerId$ = new BehaviorSubject<string | null>(null);
  chatMessages$ = new BehaviorSubject<InternalChatMessage[]>([]);

  constructor({
    scene,
    renderThread,
  }: {
    scene: MainScene;
    renderThread: RenderThread;
  }) {
    this.#scene = scene;
    this.#renderThread = renderThread;
  }

  async joinSpace(spaceId: string) {
    this.#reconnectCount = 0;

    // Fetch space publication from lens
    const { data } = await this.#lensClient
      .query<GetPublicationQuery, GetPublicationQueryVariables>(
        GetPublicationDocument,
        {
          request: { publicationId: spaceId },
        }
      )
      .toPromise();

    const publication = data?.publication as Publication | undefined;
    if (!publication) throw new Error("Space not found");

    const modelURL: string | undefined =
      publication?.metadata.media[1]?.original.url;
    if (!modelURL) throw new Error("Space model not found");

    // Create glTF entity from model URL
    const entity = new Entity();
    this.#spaceEntityId = entity.id;

    const mesh = new GLTFMesh();
    mesh.uri = modelURL;
    entity.mesh = mesh;

    // Add to scene
    await this.#scene.loadJSON({
      entities: [entity.toJSON()],
    });

    // Get host server
    const spaceHost = null; // TODO: get from metadata

    const host =
      process.env.NODE_ENV === "development"
        ? "ws://localhost:4000"
        : spaceHost
        ? `wss://${spaceHost}`
        : DEFAULT_HOST;

    this.#hostServer = host;

    // Connect to host server
    this.connectToHost(spaceId);
  }

  connectToHost(spaceId: string) {
    if (!this.#hostServer) throw new Error("No host server set");

    // Create WebSocket connection to host server
    const ws = new WebSocket(this.#hostServer);
    this.#ws = ws;

    function send(message: ToHostMessage) {
      ws.send(JSON.stringify(message));
    }

    ws.onopen = () => {
      console.info("✅ Connected to host");
      this.#reconnectCount = 0;

      // Set player name and avatar
      this.#sendName();
      this.#sendAvatar();
      this.#sendHandle();

      // Join space
      send({ subject: "join", data: { spaceId } });

      // Start broadcasting position
      this.#broadcastInterval = setInterval(() => {
        if (!this.#playerPosition || !this.#playerRotation) return;

        const data: [number, number, number, number, number, number, number] = [
          Atomics.load(this.#playerPosition, 0) / 1000,
          Atomics.load(this.#playerPosition, 1) / 1000,
          Atomics.load(this.#playerPosition, 2) / 1000,
          Atomics.load(this.#playerRotation, 0) / 100000,
          Atomics.load(this.#playerRotation, 1) / 100000,
          Atomics.load(this.#playerRotation, 2) / 100000,
          Atomics.load(this.#playerRotation, 3) / 100000,
        ];

        send({ subject: "location", data });
      }, 1000 / PUBLISH_HZ);
    };

    ws.onmessage = (event: MessageEvent<string>) => {
      const { subject, data }: FromHostMessage = JSON.parse(event.data);

      switch (subject) {
        case "join_successful": {
          // Set your name
          if (this.#myName) this.#playerNames.set(data.playerId, this.#myName);
          else this.#playerNames.delete(data.playerId);

          // Set your handle
          if (this.#myHandle)
            this.#playerHandles.set(data.playerId, this.#myHandle);
          else this.#playerHandles.delete(data.playerId);

          // Save player id
          this.playerId$.next(data.playerId);
          break;
        }

        case "player_joined": {
          console.info(`👋 Player ${data.playerId} joined`);

          // Set name
          if (data.name) this.#playerNames.set(data.playerId, data.name);
          else this.#playerNames.delete(data.playerId);

          // Set handle
          if (data.handle) this.#playerHandles.set(data.playerId, data.handle);
          else this.#playerHandles.delete(data.playerId);

          // Add player to scene
          this.#renderThread.postMessage({ subject: "player_joined", data });
          break;
        }

        case "player_left": {
          console.info(`👋 Player ${data} left`);

          // Delete name
          this.#playerNames.delete(data);

          this.#renderThread.postMessage({ subject: "player_left", data });
          break;
        }

        case "player_location": {
          this.#renderThread.postMessage({
            subject: "set_player_location",
            data,
          });
          break;
        }

        case "player_message": {
          // Get player name
          const handle = this.#playerHandles.get(data.playerId);
          const isHandle = Boolean(handle);

          let username = isHandle
            ? `@${handle}`
            : this.#playerNames.get(data.playerId);

          if (!username) username = `Guest ${data.playerId.slice(0, 4)}`;

          // Add message to chat
          const message: InternalChatMessage = {
            ...data,
            username,
            isHandle,
          };
          const newChatMessages = this.chatMessages$.value.concat(message);

          // Sort by timestamp
          newChatMessages.sort((a, b) => a.timestamp - b.timestamp);

          // Limit to 25 messages
          newChatMessages.splice(0, newChatMessages.length - 25);

          this.chatMessages$.next(newChatMessages);
          break;
        }

        case "player_falling_state": {
          this.#renderThread.postMessage({
            subject: "set_player_falling_state",
            data,
          });
          break;
        }

        case "player_name": {
          console.info(`📇 Player ${data.playerId} is now ${data.name}`);

          if (data.name) this.#playerNames.set(data.playerId, data.name);
          else this.#playerNames.delete(data.playerId);
          break;
        }

        case "player_avatar": {
          console.info(`💃 Got custom avatar for ${data.playerId}`);

          this.#renderThread.postMessage({
            subject: "set_player_avatar",
            data,
          });
          break;
        }

        case "player_handle": {
          console.info(`🌿 Player ${data.playerId} is now @${data.handle}`);

          if (data.handle) this.#playerHandles.set(data.playerId, data.handle);
          else this.#playerHandles.delete(data.playerId);
          break;
        }
      }
    };

    ws.onclose = async () => {
      console.info("❌ Disconnected from host");

      if (this.#broadcastInterval) clearInterval(this.#broadcastInterval);

      if (!this.#hostServer || this.#reconnectCount > 0) return;

      // Try reconnect
      while (this.#reconnectCount < 10) {
        const count = ++this.#reconnectCount;

        // Wait a little longer each attempt
        const timeout = Math.min(1000 * count);
        await new Promise((resolve) => setTimeout(resolve, timeout));

        // Test if has been reconnected
        if (this.#ws?.readyState === WebSocket.OPEN) return;

        // Close preview WebSocket connection
        if (this.#ws) this.#ws.close();
        this.#ws = null;

        console.info(`🔄 (${count}) Attempting reconnect to host...`);
        this.connectToHost(spaceId);
      }

      console.error("🪦 Failed to reconnect to host. Giving up.");
      this.disconnect();
    };
  }

  disconnect() {
    // Close WebSocket connection
    if (this.#ws) this.#ws.close();
    this.#ws = null;

    this.#reconnectCount = 0;
    this.#hostServer = null;
    if (this.#broadcastInterval) clearInterval(this.#broadcastInterval);
  }

  leaveSpace() {
    this.disconnect();

    // Remove space entity from scene
    if (this.#spaceEntityId) {
      this.#scene.removeEntity(this.#spaceEntityId);
      this.#spaceEntityId = null;
    }
  }

  sendChatMessage(data: string) {
    if (!this.#ws?.OPEN) return;

    const message: ToHostMessage = {
      subject: "message",
      data,
    };

    this.#ws.send(JSON.stringify(message));
  }

  setPlayerPosition(position: Int32Array) {
    this.#playerPosition = position;
  }

  setPlayerRotation(rotation: Int32Array) {
    this.#playerRotation = rotation;
  }

  setFallState(falling: boolean) {
    if (!this.#ws || !this.#isWsOpen()) return;

    const message: ToHostMessage = {
      subject: "falling_state",
      data: falling,
    };

    this.#ws.send(JSON.stringify(message));
  }

  setName(name: string | null) {
    this.#myName = name;
    this.#sendName();
  }

  #sendName() {
    if (!this.#ws || !this.#isWsOpen()) return;

    const message: ToHostMessage = {
      subject: "set_name",
      data: this.#myName,
    };

    this.#ws.send(JSON.stringify(message));
  }

  setAvatar(url: string | null) {
    this.#myAvatar = url;
    this.#renderThread.postMessage({ subject: "set_avatar", data: url });
    this.#sendAvatar();
  }

  #sendAvatar() {
    if (!this.#ws || !this.#isWsOpen()) return;

    const message: ToHostMessage = {
      subject: "set_avatar",
      data: this.#myAvatar,
    };

    this.#ws.send(JSON.stringify(message));
  }

  setHandle(handle: string | null) {
    this.#myHandle = handle;
    this.#sendHandle();
  }

  #sendHandle() {
    if (!this.#ws || !this.#isWsOpen()) return;

    const message: ToHostMessage = {
      subject: "set_handle",
      data: this.#myHandle,
    };

    this.#ws.send(JSON.stringify(message));
  }

  #isWsOpen() {
    return this.#ws && this.#ws.readyState === this.#ws.OPEN;
  }
}
