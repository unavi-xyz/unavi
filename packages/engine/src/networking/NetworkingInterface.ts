import {
  GetPublicationDocument,
  GetPublicationQuery,
  GetPublicationQueryVariables,
  Publication,
} from "lens";
import { nanoid } from "nanoid";
import { BehaviorSubject } from "rxjs";
import { createClient } from "urql";

import { MainScene } from "../main/MainScene";
import { RenderThread } from "../render/RenderThread";
import { GLTFMesh, Node } from "../scene";
import { toHex } from "../utils/toHex";
import { LENS_API } from "./constants";
import {
  FromHostMessage,
  InternalChatMessage,
  SpaceJoinStatus,
  ToHostMessage,
} from "./types";
import { WebRTC } from "./WebRTC";

/*
 * Acts as an interface for all networking functionality.
 * Handles joining spaces, sending and receiving messages, etc.
 */
export class NetworkingInterface {
  #scene: MainScene;
  #renderThread: RenderThread;

  #webRTC: WebRTC | null = null;
  #ws: WebSocket | null = null;

  #lensClient = createClient({ url: LENS_API });
  #spaceNodeId: string | null = null;
  #broadcastInterval: NodeJS.Timeout | null = null;
  #hostServer: string | null = null;
  #reconnectCount = 0;
  #producedTrack: MediaStreamTrack | null = null;

  playerPosition: Int32Array | null = null;
  playerRotation: Int16Array | null = null;

  #connectedPlayers = new Set<number>([-1]);
  #loadedPlayers = new Set<number>();
  #playerNames = new Map<number, string>();
  #playerHandles = new Map<number, string>();

  #myName: string | null = null;
  #myAvatar: string | null = null;
  #myHandle: string | null = null;

  playerId$ = new BehaviorSubject<number | null>(null);
  chatMessages$ = new BehaviorSubject<InternalChatMessage[]>([]);

  spaceJoinStatus$ = new BehaviorSubject<SpaceJoinStatus>({
    spaceId: null,
    spaceFetched: false,
    wsConnected: false,
    webrtcConnected: false,
    sceneLoaded: false,
  });

  get spaceJoinStatus() {
    return this.spaceJoinStatus$.value;
  }

  set spaceJoinStatus(status: SpaceJoinStatus) {
    this.spaceJoinStatus$.next(status);
  }

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

    this.spaceJoinStatus = {
      spaceId,
      spaceFetched: false,
      wsConnected: false,
      webrtcConnected: false,
      sceneLoaded: false,
    };

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

    this.spaceJoinStatus = {
      ...this.spaceJoinStatus,
      spaceFetched: true,
    };

    // Get host server
    const spaceHost = null; // TODO: get from metadata

    const host =
      process.env.NODE_ENV === "development"
        ? "ws://localhost:4000"
        : spaceHost
        ? `wss://${spaceHost}`
        : `wss://${process.env.NEXT_PUBLIC_DEFAULT_HOST}`;

    this.#hostServer = host;

    // Connect to host server
    this.connectToHost(spaceId);

    // Create glTF mesh
    const mesh = new GLTFMesh();
    mesh.uri = modelURL;

    const node = new Node();
    node.meshId = mesh.id;

    this.#spaceNodeId = node.id;

    // Load glTF
    await this.#scene.loadJSON(
      {
        nodes: [node.toJSON()],
        meshes: [mesh.toJSON()],
      },
      true
    );

    this.spaceJoinStatus = {
      ...this.spaceJoinStatus,
      sceneLoaded: true,
    };

    // Wait for space to load
    await new Promise((resolve, reject) => {
      const interval = setInterval(() => {
        if (this.spaceJoinStatus.spaceId !== spaceId) {
          clearInterval(interval);
          reject();
          return;
        }

        if (
          this.spaceJoinStatus.wsConnected &&
          this.spaceJoinStatus.webrtcConnected &&
          this.#loadedPlayers.size === this.#connectedPlayers.size
        ) {
          clearInterval(interval);
          resolve(null);
        }
      }, 100);
    });
  }

  connectToHost(spaceId: string) {
    if (!this.#hostServer) throw new Error("No host server set");

    // Create WebSocket connection to host server
    const ws = new WebSocket(this.#hostServer);
    this.#ws = ws;

    // Create WebRTC manager
    this.#webRTC = new WebRTC(
      ws,
      this,
      this.#renderThread,
      this.#producedTrack
    );
    this.#webRTC.playerId = this.playerId$.value;
    this.#webRTC.playerPosition = this.playerPosition;
    this.#webRTC.playerRotation = this.playerRotation;

    function send(message: ToHostMessage) {
      ws.send(JSON.stringify(message));
    }

    ws.onopen = () => {
      console.info("✅ Connected to host");
      this.#reconnectCount = 0;

      this.spaceJoinStatus = {
        ...this.spaceJoinStatus,
        wsConnected: true,
      };

      // Start WebRTC connection
      if (!this.#webRTC) throw new Error("WebRTC not initialized");
      this.#webRTC.connect();

      // Join space
      send({ subject: "join", data: { spaceId } });
    };

    ws.onmessage = (event: MessageEvent<string>) => {
      this.#webRTC?.onmessage(JSON.parse(event.data));

      const { subject, data }: FromHostMessage = JSON.parse(event.data);

      switch (subject) {
        case "join_successful": {
          // Send player name and avatar
          if (this.#myName) this.#sendName();
          if (this.#myAvatar) this.#sendAvatar();
          if (this.#myHandle) this.#sendHandle();

          // Set your name
          if (this.#myName) this.#playerNames.set(data.playerId, this.#myName);
          else this.#playerNames.delete(data.playerId);

          // Set your handle
          if (this.#myHandle)
            this.#playerHandles.set(data.playerId, this.#myHandle);
          else this.#playerHandles.delete(data.playerId);

          // Save player id
          this.playerId$.next(data.playerId);
          if (this.#webRTC) this.#webRTC.playerId = data.playerId;

          break;
        }

        case "player_joined": {
          console.info(`👋 Player ${toHex(data.playerId)} joined`);

          this.#connectedPlayers.add(data.playerId);

          // Set name
          if (data.name) this.#playerNames.set(data.playerId, data.name);
          else this.#playerNames.delete(data.playerId);

          // Set handle
          if (data.handle) this.#playerHandles.set(data.playerId, data.handle);
          else this.#playerHandles.delete(data.playerId);

          // Add player to scene
          this.#renderThread.postMessage({ subject: "player_joined", data });

          // Get player name
          const username = this.#getUsername(data.playerId);

          this.#renderThread.postMessage({
            subject: "player_name",
            data: {
              playerId: data.playerId,
              name: username,
            },
          });

          const handle = this.#playerHandles.get(data.playerId);
          const isHandle = Boolean(handle);

          // Add message to chat if they joined after you
          if (!data.beforeYou)
            this.#addChatMessage({
              type: "system",
              variant: "player_joined",
              id: nanoid(),
              timestamp: Date.now(),
              playerId: data.playerId,
              username,
              isHandle,
            });

          break;
        }

        case "player_left": {
          console.info(`👋 Player ${toHex(data)} left`);

          this.#connectedPlayers.delete(data);
          this.#loadedPlayers.delete(data);

          // Delete name
          this.#playerNames.delete(data);

          this.#renderThread.postMessage({ subject: "player_left", data });

          // Get player name
          const username = this.#getUsername(data);

          const handle = this.#playerHandles.get(data);
          const isHandle = Boolean(handle);

          // Add message to chat
          this.#addChatMessage({
            type: "system",
            variant: "player_left",
            id: nanoid(),
            timestamp: Date.now(),
            playerId: data,
            username,
            isHandle,
          });
          break;
        }

        case "player_message": {
          // Get player name
          const username = this.#getUsername(data.playerId);

          const handle = this.#playerHandles.get(data.playerId);
          const isHandle = Boolean(handle);

          // Add message to chat
          this.#addChatMessage({
            type: "chat",
            username,
            isHandle,
            ...data,
          });
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
          console.info(`📇 Player ${toHex(data.playerId)} is now ${data.name}`);

          if (data.name) this.#playerNames.set(data.playerId, data.name);
          else this.#playerNames.delete(data.playerId);

          const username = this.#getUsername(data.playerId);

          this.#renderThread.postMessage({
            subject: "player_name",
            data: {
              playerId: data.playerId,
              name: username,
            },
          });

          break;
        }

        case "player_avatar": {
          console.info(`💃 Got custom avatar for ${toHex(data.playerId)}`);

          this.#loadedPlayers.delete(data.playerId);

          this.#renderThread.postMessage({
            subject: "set_player_avatar",
            data,
          });
          break;
        }

        case "player_handle": {
          console.info(
            `🌿 Player ${toHex(data.playerId)} is now @${data.handle}`
          );

          if (data.handle) this.#playerHandles.set(data.playerId, data.handle);
          else this.#playerHandles.delete(data.playerId);
          break;
        }
      }
    };

    ws.onclose = async () => {
      console.info("❌ Disconnected from host");

      // Remove all players from scene
      this.#playerHandles.clear();
      this.#playerNames.clear();
      this.#renderThread.postMessage({ subject: "clear_players", data: null });

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
    // Close WebRTC connection
    if (this.#webRTC) this.#webRTC.disconnect();

    // Close WebSocket connection
    if (this.#ws) this.#ws.close();
    this.#ws = null;

    this.#reconnectCount = 0;
    this.#hostServer = null;
    if (this.#broadcastInterval) clearInterval(this.#broadcastInterval);
  }

  leaveSpace() {
    this.disconnect();

    // Remove space node from scene
    if (this.#spaceNodeId) {
      this.#scene.removeNode(this.#spaceNodeId);
      this.#spaceNodeId = null;
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
    this.playerPosition = position;
    if (this.#webRTC) this.#webRTC.playerPosition = position;
  }

  setPlayerRotation(rotation: Int16Array) {
    this.playerRotation = rotation;
    if (this.#webRTC) this.#webRTC.playerRotation = rotation;
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

  #addChatMessage(message: InternalChatMessage) {
    const newChatMessages = this.chatMessages$.value.concat(message);

    // Sort by timestamp
    newChatMessages.sort((a, b) => a.timestamp - b.timestamp);

    // Limit to 25 messages
    newChatMessages.splice(0, newChatMessages.length - 25);

    this.chatMessages$.next(newChatMessages);
  }

  #getUsername(playerId: number) {
    const handle = this.#playerHandles.get(playerId);
    const isHandle = Boolean(handle);

    let username = isHandle ? `@${handle}` : this.#playerNames.get(playerId);
    if (!username) username = `Guest ${toHex(playerId)}`;

    return username;
  }

  #isWsOpen() {
    return this.#ws && this.#ws.readyState === this.#ws.OPEN;
  }

  produceAudio(track: MediaStreamTrack) {
    if (!this.#webRTC) throw new Error("WebRTC not initialized");
    this.#producedTrack = track;
    return this.#webRTC.produceAudio(track);
  }

  setAudioPaused(paused: boolean) {
    if (!this.#webRTC) throw new Error("WebRTC not initialized");
    this.#webRTC.setAudioPaused(paused);
  }

  setPlayerLoaded(playerId: number) {
    if (this.#connectedPlayers.has(playerId)) this.#loadedPlayers.add(playerId);
  }
}
