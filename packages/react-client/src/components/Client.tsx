import { ToHostMessage } from "@wired-labs/protocol";
import { ERC721Metadata } from "contracts";
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

export type HoverState = null | "avatar";

export interface IClientContext {
  engine: Engine | null;
  spaceId: number | null;
  ethersProvider: providers.Provider | Signer | null;

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
  engine: null,
  spaceId: null,
  ethersProvider: null,

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
  spaceId?: number;
  metadata?: ERC721Metadata;
}

/**
 * A self-contained client for connecting to the Wired.
 *
 * @param animations The path to the animations folder.
 * @param children Any children to render.
 * @param defaultAvatar The path to the default avatar.
 * @param ethers The ethers provider to use.
 * @param host The host to connect to.
 * @param skybox The path to the skybox.
 * @param spaceId The space to connect to.
 * @param metadata The metadata for the space.
 */
export function Client({
  animations,
  children,
  defaultAvatar,
  ethers,
  host = "ws://localhost:4000",
  metadata,
  skybox,
  spaceId,
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
    send({ subject: "set_avatar", data: avatar });
    engine.render.send({ subject: "set_user_avatar", data: avatar });
  }, [avatar, engine, send]);

  return (
    <ClientContext.Provider
      value={{
        engine,
        spaceId: spaceId ?? null,
        ethersProvider,
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

      {spaceId !== undefined && metadata ? (
        <Space spaceId={spaceId} metadata={metadata} host={host} />
      ) : null}

      {children}
    </ClientContext.Provider>
  );
}

interface SpaceProps {
  spaceId: number;
  metadata: ERC721Metadata;
  host: string;
}

function Space({ spaceId, metadata, host }: SpaceProps) {
  const { setLoadingProgress, setLoadingText } = useContext(ClientContext);
  const { loadingProgress, loadingText } = useSpace(spaceId, metadata, host);

  useEffect(() => {
    setLoadingProgress(loadingProgress);
  }, [loadingProgress, setLoadingProgress]);

  useEffect(() => {
    setLoadingText(loadingText);
  }, [loadingText, setLoadingText]);

  return null;
}
