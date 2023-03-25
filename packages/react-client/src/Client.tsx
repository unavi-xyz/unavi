import { ERC721Metadata } from "contracts";
import { Engine } from "engine";
import {
  createContext,
  Dispatch,
  SetStateAction,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";

import { Player } from "./classes/Player";
import { useResizeCanvas } from "./hooks/useResizeCanvas";
import { useSpace } from "./hooks/useSpace";

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

interface IClientContext {
  engine: Engine | null;
  spaceId: number | null;

  playerId: number | null;
  setPlayerId: Dispatch<SetStateAction<number | null>>;

  ws: WebSocket | null;
  setWs: Dispatch<SetStateAction<WebSocket | null>>;

  loadingProgress: number;
  loadingText: string;
  setLoadingProgress: (progress: number) => void;
  setLoadingText: (text: string) => void;

  players: Player[];
  chatMessages: ChatMessage[];
  setPlayers: Dispatch<SetStateAction<Player[]>>;
  setChatMessages: Dispatch<SetStateAction<ChatMessage[]>>;

  micTrack: MediaStreamTrack | null;
  setMicTrack: (track: MediaStreamTrack | null) => void;
  micEnabled: boolean;
  setMicEnabled: (enabled: boolean) => void;
}

const defaultContext: IClientContext = {
  engine: null,
  spaceId: null,

  playerId: null,
  setPlayerId: () => {},

  ws: null,
  setWs: () => {},

  loadingProgress: 0,
  loadingText: "",
  setLoadingProgress: () => {},
  setLoadingText: () => {},

  players: [],
  chatMessages: [],
  setPlayers: () => {},
  setChatMessages: () => {},

  micTrack: null,
  setMicTrack: () => {},
  micEnabled: false,
  setMicEnabled: () => {},
};

export const ClientContext = createContext<IClientContext>(defaultContext);

interface Props {
  animations?: string;
  avatar?: string;
  children?: React.ReactNode;
  host?: string;
  skybox?: string;
  spaceId?: number;
  metadata?: ERC721Metadata;
}

/**
 * A self-contained client for connecting to the Wired.
 *
 * @param animations The path to the animations folder.
 * @param avatar The path to the default avatar.
 * @param children Any children to render.
 * @param host The host to connect to.
 * @param skybox The path to the skybox.
 * @param spaceId The space to connect to.
 * @param metadata The metadata for the space.
 */
export function Client({
  animations,
  avatar,
  children,
  host = "ws://localhost:4000",
  metadata,
  skybox,
  spaceId,
}: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);

  const [engine, setEngine] = useState<Engine | null>(defaultContext.engine);
  const [micEnabled, setMicEnabled] = useState<boolean>(defaultContext.micEnabled);
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [loadingProgress, setLoadingProgress] = useState(0);
  const [loadingText, setLoadingText] = useState("");
  const [players, setPlayers] = useState<Player[]>([]);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [micTrack, setMicTrack] = useState<MediaStreamTrack | null>(null);
  const [playerId, setPlayerId] = useState<number | null>(null);

  useResizeCanvas(engine, canvasRef, overlayRef, containerRef);

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
    if (!engine || !avatar) return;
    engine.render.send({ subject: "set_default_avatar", data: avatar });
  }, [engine, avatar]);

  useEffect(() => {
    if (!engine || !skybox) return;
    engine.render.send({ subject: "set_skybox", data: { uri: skybox } });
  }, [engine, skybox]);

  return (
    <ClientContext.Provider
      value={{
        engine,
        spaceId: spaceId ?? null,
        playerId,
        setPlayerId,
        ws,
        setWs,
        micTrack,
        setMicTrack,
        micEnabled,
        setMicEnabled,
        loadingProgress,
        loadingText,
        setLoadingProgress,
        setLoadingText,
        players,
        chatMessages,
        setPlayers,
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
