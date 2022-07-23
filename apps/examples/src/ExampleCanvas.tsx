import { useEffect, useRef, useState } from "react";
import { AnimationAction, Scene } from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader";

import { Engine } from "@wired-xr/new-engine";

import Panel from "./Panel/Panel";

export interface RenderInfo {
  load: {
    time: number;
    threeTime: number;
  };
  memory: {
    geometries: number;
    textures: number;
  };
  render: {
    calls: number;
    lines: number;
    points: number;
    triangles: number;
  };
}

interface Props {
  gltf: string;
}

export default function ExampleCanvas({ gltf }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const [loaded, setLoaded] = useState(false);
  const [animations, setAnimations] = useState<AnimationAction[]>();
  const [info, setInfo] = useState<RenderInfo>();

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

    let interval: NodeJS.Timer;
    const startTime = performance.now();

    // Load gltf
    engine
      .loadGltf(gltf)
      .then(async ({ animations }) => {
        const loadTime = Math.round(performance.now() - startTime) / 1000;

        setAnimations(animations);
        setLoaded(true);

        // Start first animation
        if (animations.length > 0) {
          animations[0].play();
        }

        // Test against threejs loader load time
        const threeLoader = new GLTFLoader();
        const startThreeTime = performance.now();
        await threeLoader.loadAsync(gltf);
        const threeTime = Math.round(performance.now() - startThreeTime) / 1000;

        function updateInfo() {
          const { render, memory } = engine.info();
          const { triangles, points, lines, calls } = render;

          setInfo({
            load: {
              time: loadTime,
              threeTime,
            },
            memory,
            render: {
              calls,
              lines,
              points,
              triangles,
            },
          });
        }

        updateInfo();

        interval = setInterval(updateInfo, 1000);
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
      setInfo(undefined);
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

      <Panel loaded={loaded} animations={animations} info={info} />
    </>
  );
}
