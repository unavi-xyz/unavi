import { useResizeCanvas } from "@unavi/react-client";
import React, { useCallback, useEffect, useRef } from "react";

import SignInButton from "@/app/(navbar)/SignInButton";
import Crosshair from "@/src/play/Crosshair";

import { ERROR_NOT_SIGNED_IN } from "../hooks/useLoad";
import { ERROR_MESSAGE } from "../utils/parseError";
import { useEditor } from "./Editor";

interface Props {
  setResize: (resize: () => void) => void;
}

export default function EditorCanvas({ setResize }: Props) {
  const overlayRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const { canvasRef, error, engine, mode, loaded } = useEditor();
  const resize = useResizeCanvas(engine);

  const onResize = useCallback(() => {
    if (!containerRef.current) return;
    resize(containerRef.current.clientWidth, containerRef.current.clientHeight);
  }, [resize]);

  useEffect(() => {
    setResize(onResize);
  }, [onResize, setResize]);

  useEffect(() => {
    if (!engine) return;

    // Convert to OffscreenCanvas if supported, or if we're in development mode
    if (
      canvasRef.current &&
      (typeof OffscreenCanvas === "undefined" || process.env.NODE_ENV === "development")
    ) {
      const offscreen = canvasRef.current.transferControlToOffscreen();
      engine.canvas = offscreen;
    } else {
      engine.canvas = canvasRef.current;
    }

    engine.overlayCanvas = overlayRef.current;
  }, [engine, canvasRef]);

  // Resize canvas on window resize
  useEffect(() => {
    onResize();

    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, [onResize]);

  if (error) {
    return (
      <div className="h-full space-y-2 bg-neutral-100 pt-10 text-center">
        {error === ERROR_NOT_SIGNED_IN ? (
          <h2>{error}</h2>
        ) : error === ERROR_MESSAGE.UNAUTHORIZED ? (
          <h2>Project not found.</h2>
        ) : (
          <h2>Failed to load project. {error}</h2>
        )}

        {error === ERROR_NOT_SIGNED_IN ? (
          <SignInButton />
        ) : (
          <button
            onClick={() => window.location.reload()}
            className="rounded-lg border border-neutral-500 px-4 py-1 hover:bg-neutral-200 active:bg-neutral-300"
          >
            Try again
          </button>
        )}
      </div>
    );
  }

  return (
    <div className={`h-full bg-neutral-300 ${loaded ? "" : "animate-pulse"}`}>
      <div
        ref={containerRef}
        className={`relative h-full w-full overflow-hidden transition ${
          loaded ? "opacity-100" : "opacity-0"
        }`}
      >
        {mode === "play" ? <Crosshair /> : null}

        <canvas ref={canvasRef} className="h-full w-full" />
        <canvas ref={overlayRef} className="absolute top-0 left-0 z-10 h-full w-full" />
      </div>
    </div>
  );
}
