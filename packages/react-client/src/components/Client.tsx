import { RequestMessage, WorldMetadata } from "@wired-protocol/types";
import { Engine, EngineOptions } from "engine";
import {
  createContext,
  Dispatch,
  SetStateAction,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";

import { Player } from "../classes/Player";
import { LocalStorageKey } from "../constants";
import { useAvatarEquip } from "../hooks/useAvatarEquip";
import { useSpace } from "../hooks/useSpace";
import ClientCanvas from "./ClientCanvas";

export type SystemMessage = {
  type: "system";
  variant: "player_joined" | "player_left";
  playerId: number;
  id: string;
  timestamp: number;
};

export type PlayerMessage = {
  type: "player";
  text: string;
  playerId: number;
  id: string;
  timestamp: number;
};

export type ChatMessage = SystemMessage | PlayerMessage;

export type HoverState = null | "equip_avatar" | "avatar_equipped";

export interface IClientContext {
  host: string | null;
  uri: string | null;
  baseHomeServer: string | null;

  engine: Engine | null;

  hoverState: HoverState;
  setHoverState: Dispatch<SetStateAction<HoverState>>;

  ws: WebSocket | null;
  setWs: Dispatch<SetStateAction<WebSocket | null>>;
  send: (message: RequestMessage) => void;

  playerId: number | null;
  setPlayerId: Dispatch<SetStateAction<number | null>>;
  avatar: string | null;
  setAvatar: (avatar: string | null) => void;

  loadingProgress: number;
  setLoadingProgress: (progress: number) => void;
  loadingText: string;
  setLoadingText: (text: string) => void;

  players: Player[];
  setPlayers: Dispatch<SetStateAction<Player[]>>;
  chatMessages: ChatMessage[];
  setChatMessages: Dispatch<SetStateAction<ChatMessage[]>>;

  micTrack: MediaStreamTrack | null;
  setMicTrack: (track: MediaStreamTrack | null) => void;
  micEnabled: boolean;
  setMicEnabled: (enabled: boolean) => void;
}

const defaultContext: IClientContext = {
  avatar: null,
  baseHomeServer: null,
  chatMessages: [],

  engine: null,

  host: null,
  hoverState: null,

  loadingProgress: 0,
  loadingText: "",
  micEnabled: false,

  micTrack: null,
  playerId: null,
  players: [],
  send: () => {},

  setAvatar: () => {},
  setChatMessages: () => {},
  setHoverState: () => {},
  setLoadingProgress: () => {},

  setLoadingText: () => {},
  setMicEnabled: () => {},
  setMicTrack: () => {},
  setPlayerId: () => {},

  setPlayers: () => {},
  setWs: () => {},
  uri: null,
  ws: null,
};

export const ClientContext = createContext<IClientContext>(defaultContext);

interface Props {
  animations?: string;
  children?: React.ReactNode;
  defaultAvatar?: string;
  host?: string;
  skybox?: string;
  uri?: string;
  baseHomeServer?: string;
  metadata?: WorldMetadata;
  engineOptions?: EngineOptions;
}

/**
 * A self-contained client for connecting to UNAVI.
 */
export function Client({
  animations,
  children,
  baseHomeServer,
  defaultAvatar,
  engineOptions,
  host,
  metadata,
  skybox,
  uri,
}: Props) {
  const [avatar, setAvatar] = useState<string | null>(null);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [engine, setEngine] = useState<Engine | null>(null);
  const [hoverState, setHoverState] = useState<HoverState>(defaultContext.hoverState);
  const [loadingProgress, setLoadingProgress] = useState(0);
  const [loadingText, setLoadingText] = useState("");
  const [micEnabled, setMicEnabled] = useState<boolean>(defaultContext.micEnabled);
  const [micTrack, setMicTrack] = useState<MediaStreamTrack | null>(null);
  const [playerId, setPlayerId] = useState<number | null>(null);
  const [players, setPlayers] = useState<Player[]>([]);
  const [ws, setWs] = useState<WebSocket | null>(null);

  useAvatarEquip(engine, avatar, setAvatar, setHoverState);

  const worldHost = metadata?.info?.host;
  const usedHost = host || worldHost;

  const send = useCallback(
    (message: RequestMessage) => {
      if (!ws || ws.readyState !== ws.OPEN) return;
      ws.send(JSON.stringify(message));
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [ws, ws?.readyState]
  );

  useEffect(() => {
    const newEngine = new Engine(engineOptions);

    newEngine.render.send({ data: true, subject: "toggle_animations" });
    newEngine.start();

    setEngine(newEngine);

    return () => {
      newEngine.destroy();
      setEngine(null);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!engine) return;
    engine.render.send({ data: animations ?? null, subject: "set_animations_path" });
  }, [engine, animations]);

  useEffect(() => {
    if (!engine) return;
    engine.render.send({ data: defaultAvatar ?? null, subject: "set_default_avatar" });
  }, [engine, defaultAvatar]);

  useEffect(() => {
    if (!engine) return;
    engine.render.send({ data: skybox ?? null, subject: "set_skybox" });
  }, [engine, skybox]);

  useEffect(() => {
    if (!engine) return;

    // Send to host
    send({ data: avatar, id: "xyz.unavi.world.user.avatar" });

    // Send to engine
    engine.render.send({ data: avatar, subject: "set_user_avatar" });

    // Save to local storage
    if (avatar) localStorage.setItem(LocalStorageKey.Avatar, avatar);
    else localStorage.removeItem(LocalStorageKey.Avatar);
  }, [avatar, engine, send]);

  // Load avatar from local storage
  useEffect(() => {
    const localAvatar = localStorage.getItem(LocalStorageKey.Avatar);
    if (localAvatar) setAvatar(localAvatar);
  }, []);

  const value = useMemo(
    () => ({
      avatar,
      baseHomeServer: baseHomeServer || null,
      chatMessages,
      engine,
      host: usedHost || null,
      hoverState,
      loadingProgress,
      loadingText,
      micEnabled,
      micTrack,
      playerId,
      players,
      send,
      setAvatar,
      setChatMessages,
      setHoverState,
      setLoadingProgress,
      setLoadingText,
      setMicEnabled,
      setMicTrack,
      setPlayerId,
      setPlayers,
      setWs,
      uri: uri || null,
      ws,
    }),
    [
      usedHost,
      uri,
      baseHomeServer,
      engine,
      hoverState,
      ws,
      send,
      playerId,
      avatar,
      micTrack,
      micEnabled,
      loadingProgress,
      loadingText,
      players,
      chatMessages,
    ]
  );

  return (
    <ClientContext.Provider value={value}>
      <ClientCanvas engine={engine} />

      <Space />

      {children}
    </ClientContext.Provider>
  );
}

function Space() {
  const { uri, host, setLoadingProgress, setLoadingText } = useContext(ClientContext);
  const { loadingProgress, loadingText } = useSpace(uri, host);

  useEffect(() => {
    setLoadingProgress(loadingProgress);
  }, [loadingProgress, setLoadingProgress]);

  useEffect(() => {
    setLoadingText(loadingText);
  }, [loadingText, setLoadingText]);

  return null;
}
