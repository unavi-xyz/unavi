import { ToHostMessage } from "@wired-labs/protocol";
import { ERC721Metadata, getHostFromMetadata } from "contracts";
import { Engine } from "engine";
import { providers, Signer } from "ethers";
import {
  createContext,
  Dispatch,
  SetStateAction,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";

import { Player } from "../classes/Player";
import { LocalStorageKey } from "../constants";
import { useAvatarEquip } from "../hooks/useAvatarEquip";
import { useResizeCanvas } from "../hooks/useResizeCanvas";
import { useSpace } from "../hooks/useSpace";

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
  ethersProvider: providers.Provider | Signer | null;
  host: string | null;
  uri: string | null;

  engine: Engine | null;

  hoverState: HoverState;
  setHoverState: Dispatch<SetStateAction<HoverState>>;

  ws: WebSocket | null;
  setWs: Dispatch<SetStateAction<WebSocket | null>>;
  send: (message: ToHostMessage) => void;

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
  ethersProvider: null,
  host: null,
  uri: null,

  engine: null,

  hoverState: null,
  setHoverState: () => {},

  ws: null,
  setWs: () => {},
  send: () => {},

  playerId: null,
  setPlayerId: () => {},
  avatar: null,
  setAvatar: () => {},

  loadingProgress: 0,
  setLoadingProgress: () => {},
  loadingText: "",
  setLoadingText: () => {},

  players: [],
  setPlayers: () => {},
  chatMessages: [],
  setChatMessages: () => {},

  micTrack: null,
  setMicTrack: () => {},
  micEnabled: false,
  setMicEnabled: () => {},
};

export const ClientContext = createContext<IClientContext>(defaultContext);

interface Props {
  animations?: string;
  children?: React.ReactNode;
  defaultAvatar?: string;
  ethers?: providers.Provider | Signer;
  host?: string;
  skybox?: string;
  uri?: string;
  metadata?: ERC721Metadata;
}

/**
 * A self-contained client for connecting to The Wired.
 */
export function Client({
  animations,
  children,
  defaultAvatar,
  ethers,
  host,
  metadata,
  skybox,
  uri,
}: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);

  const [avatar, setAvatar] = useState<string | null>(null);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [engine, setEngine] = useState<Engine | null>(defaultContext.engine);
  const [ethersProvider, setEthersProvider] = useState<providers.Provider | Signer | null>(
    defaultContext.ethersProvider
  );
  const [hoverState, setHoverState] = useState<HoverState>(defaultContext.hoverState);
  const [loadingProgress, setLoadingProgress] = useState(0);
  const [loadingText, setLoadingText] = useState("");
  const [micEnabled, setMicEnabled] = useState<boolean>(defaultContext.micEnabled);
  const [micTrack, setMicTrack] = useState<MediaStreamTrack | null>(null);
  const [playerId, setPlayerId] = useState<number | null>(null);
  const [players, setPlayers] = useState<Player[]>([]);
  const [ws, setWs] = useState<WebSocket | null>(null);

  useResizeCanvas(engine, canvasRef, overlayRef, containerRef);
  useAvatarEquip(engine, avatar, setAvatar, setHoverState);

  const spaceHost = metadata ? getHostFromMetadata(metadata) : "";
  const usedHost = host || spaceHost;

  const send = useCallback(
    (message: ToHostMessage) => {
      if (!ws || ws.readyState !== ws.OPEN) return;
      ws.send(JSON.stringify(message));
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [ws, ws?.readyState]
  );

  useEffect(() => {
    if (!ethers) {
      setEthersProvider(null);
      return;
    }

    setEthersProvider(ethers);
  }, [ethers]);

  useEffect(() => {
    if (!canvasRef.current || !overlayRef.current) return;

    const newEngine = new Engine({ canvas: canvasRef.current, overlayCanvas: overlayRef.current });
    newEngine.render.send({ subject: "toggle_animations", data: true });
    newEngine.start();
    setEngine(newEngine);

    return () => {
      newEngine.destroy();
      setEngine(null);
    };
  }, []);

  useEffect(() => {
    if (!engine || !animations) return;
    engine.render.send({ subject: "set_animations_path", data: animations });
  }, [engine, animations]);

  useEffect(() => {
    if (!engine || !defaultAvatar) return;
    engine.render.send({ subject: "set_default_avatar", data: defaultAvatar });
  }, [engine, defaultAvatar]);

  useEffect(() => {
    if (!engine || !skybox) return;
    engine.render.send({ subject: "set_skybox", data: { uri: skybox } });
  }, [engine, skybox]);

  useEffect(() => {
    if (!engine) return;

    // Send to host
    send({ subject: "set_avatar", data: avatar });

    // Send to engine
    engine.render.send({ subject: "set_user_avatar", data: avatar });

    // Save to local storage
    if (avatar) localStorage.setItem(LocalStorageKey.Avatar, avatar);
    else localStorage.removeItem(LocalStorageKey.Avatar);
  }, [avatar, engine, send]);

  // Load avatar from local storage
  useEffect(() => {
    const localAvatar = localStorage.getItem(LocalStorageKey.Avatar);
    if (localAvatar) setAvatar(localAvatar);
  }, []);

  return (
    <ClientContext.Provider
      value={{
        ethersProvider,
        host: usedHost || null,
        uri: uri || null,
        engine,
        hoverState,
        setHoverState,
        ws,
        setWs,
        send,
        playerId,
        setPlayerId,
        avatar,
        setAvatar,
        micTrack,
        setMicTrack,
        micEnabled,
        setMicEnabled,
        loadingProgress,
        setLoadingProgress,
        loadingText,
        setLoadingText,
        players,
        setPlayers,
        chatMessages,
        setChatMessages,
      }}
    >
      <div
        ref={containerRef}
        style={{
          position: "relative",
          width: "100%",
          height: "100%",
          overflow: "hidden",
        }}
      >
        <canvas
          ref={canvasRef}
          style={{
            width: "100%",
            height: "100%",
          }}
        />
        <canvas
          ref={overlayRef}
          style={{
            position: "absolute",
            inset: 0,
            zIndex: 10,
            width: "100%",
            height: "100%",
          }}
        />
      </div>

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
