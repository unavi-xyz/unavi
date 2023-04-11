"use client";

import { Engine } from "engine";
import Script from "next/script";
import React, {
  createContext,
  Dispatch,
  SetStateAction,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

import { Project } from "@/src/server/helpers/fetchProject";

import { useAutosave } from "../hooks/useAutosave";
import { useLoad } from "../hooks/useLoad";
import { useTransformControls } from "../hooks/useTransformControls";

export type EditorMode = "edit" | "play";
export type Tool = "translate" | "rotate" | "scale";

interface EditorContextType {
  canvasRef: React.RefObject<HTMLCanvasElement>;
  changeMode: (value: EditorMode) => Promise<void>;
  engine: Engine | null;
  error: string | null;
  image: string | null;
  loaded: boolean;
  mode: EditorMode;
  scriptId: string | null;
  selectedId: string | null;
  setImage: Dispatch<SetStateAction<string | null>>;
  setScriptId: Dispatch<SetStateAction<string | null>>;
  setSelectedId: Dispatch<SetStateAction<string | null>>;
  setTitle: Dispatch<SetStateAction<string>>;
  setTool: Dispatch<SetStateAction<Tool>>;
  title: string;
  tool: Tool;
}

const defaultContext: EditorContextType = {
  canvasRef: React.createRef(),
  changeMode: async () => {},
  engine: null,
  error: null,
  image: null,
  loaded: false,
  mode: "edit",
  scriptId: null,
  selectedId: null,
  setImage: () => {},
  setScriptId: () => {},
  setSelectedId: () => {},
  setTitle: () => {},
  setTool: () => {},
  title: "",
  tool: "translate",
};

export const EditorContext = createContext<EditorContextType>(defaultContext);

export function useEditor() {
  return React.useContext(EditorContext);
}

interface Props {
  project: Project;
  animationPath?: string;
  defaultAvatar?: string;
  skybox?: string;
  children?: React.ReactNode;
}

export function Editor({ project, animationPath, defaultAvatar, skybox, children }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const sceneRef = useRef<Uint8Array | null>(null);
  const hasSceneRef = useRef(false);

  const [engine, setEngine] = useState<Engine | null>(null);
  const [image, setImage] = useState<string | null>(project.image);
  const [mode, setMode] = useState<EditorMode>("edit");
  const [scriptId, setScriptId] = useState<string | null>(null);
  const [scriptsReady, setScriptsReady] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [title, setTitle] = useState(project.title);
  const [tool, setTool] = useState<Tool>("translate");

  const { error, loaded, load } = useLoad(engine);
  useTransformControls(engine, selectedId, setSelectedId, mode, tool);
  useAutosave(project, mode);

  useEffect(() => {
    const newEngine = new Engine();
    setEngine(newEngine);

    newEngine.controls = "orbit";
    newEngine.start();

    return () => {
      newEngine.destroy();
    };
  }, []);

  useEffect(() => {
    if (!engine) return;
    engine.render.send({ subject: "set_animations_path", data: animationPath ?? null });
  }, [engine, animationPath]);

  useEffect(() => {
    if (!engine) return;
    engine.render.send({ subject: "set_default_avatar", data: defaultAvatar ?? null });
  }, [engine, defaultAvatar]);

  useEffect(() => {
    if (!engine) return;
    engine.render.send({ subject: "set_skybox", data: skybox ?? null });
  }, [engine, skybox]);

  useEffect(() => {
    if (!engine || !scriptsReady) return;

    load(project);

    return () => {
      engine.scene.clear();
    };
  }, [engine, scriptsReady, load, project]);

  const changeMode = useCallback(
    async (value: EditorMode) => {
      if (!engine || !loaded) return;

      if (value === "edit") {
        // Exit play mode
        engine.audio.stop();
        engine.behavior.stop();
        engine.controls = "orbit";
        engine.render.send({ subject: "toggle_animations", data: false });

        // Load scene
        if (hasSceneRef.current) {
          engine.scene.clear();
          if (sceneRef.current) await engine.scene.addBinary(sceneRef.current);
          hasSceneRef.current = false;
        }
      } else if (value === "play") {
        // Export scene
        const glb = await engine.scene.export();
        sceneRef.current = glb;
        hasSceneRef.current = true;

        // Enter play mode
        engine.controls = "player";
        engine.physics.send({ subject: "respawn", data: null });
        engine.render.send({ subject: "toggle_animations", data: true });
        engine.behavior.start();
        await engine.audio.start();
      }

      setMode(value);
    },
    [engine, loaded]
  );

  const value = useMemo(
    () => ({
      canvasRef,
      changeMode,
      engine,
      error,
      image,
      loaded,
      mode,
      scriptId,
      selectedId,
      setImage,
      setScriptId,
      setSelectedId,
      setTitle,
      setTool,
      title,
      tool,
    }),
    [canvasRef, changeMode, engine, error, image, loaded, mode, scriptId, selectedId, title, tool]
  );

  return (
    <>
      <Script src="/scripts/draco_decoder.js" onReady={() => setScriptsReady(true)} />
      <Script src="/scripts/draco_encoder.js" />

      <EditorContext.Provider value={value}>{children}</EditorContext.Provider>
    </>
  );
}
