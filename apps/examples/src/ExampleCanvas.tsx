import { useEffect, useRef, useState } from "react";
import { AnimationAction } from "three";

import { Engine } from "@wired-xr/new-engine";

import Panel from "./Panel/Panel";

interface Props {
  gltf: string;
}

export default function ExampleCanvas({ gltf }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const [animations, setAnimations] = useState<AnimationAction[]>();
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    if (!canvasRef.current) return;

    // Set canvas size
    updateCanvasSize();
    window.addEventListener("resize", updateCanvasSize);

    // Create engine
    const engine = new Engine({ canvas: canvasRef.current });

    // Move stats to the right
    const stats = document.getElementById("stats");
    if (stats) {
      stats.style.left = "calc(100% - 79px)";
    }

    let interval: any;

    // Load gltf
    engine
      .loadGltf(gltf)
      .then((res) => {
        const { scene, animations } = res;
        setAnimations(animations);
        setLoaded(true);

        // Start first animation
        if (animations.length > 0) {
          animations[0].play();
        }

        // console.log("🧛‍♀️", animations[0]);

        const targetUUID = animations[0].getClip().tracks[0].name.split(".")[0];

        scene.traverse((child) => {
          if (child.uuid !== targetUUID) return;

          // console.log("🧳", child);

          interval = setInterval(() => {
            // console.log(child.rotation);
          }, 810);
        });
      })
      .catch((error) => {
        console.error(error);
        setLoaded(false);
      });

    return () => {
      engine.destroy();
      window.removeEventListener("resize", updateCanvasSize);
      setAnimations(undefined);
      setLoaded(false);
      clearInterval(interval);
    };
  }, [canvasRef, gltf]);

  function updateCanvasSize() {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const container = containerRef.current;
    if (!container) return;

    // Resize canvas
    canvas.width = container.clientWidth;
    canvas.height = container.clientHeight;
  }

  return (
    <>
      <div ref={containerRef} className="relative w-full h-full overflow-hidden">
        <canvas ref={canvasRef} className="w-full h-full" />
      </div>

      <Panel loaded={loaded} animations={animations} />
    </>
  );
}
