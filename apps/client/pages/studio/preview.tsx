import { Canvas } from "@react-three/fiber";
import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect } from "react";
import { MdClose } from "react-icons/md";

import { InstancedScene } from "@wired-xr/scene";

import Player from "../../src/components/app/Player";
import { useProject } from "../../src/helpers/studio/hooks/useProject";
import { useStudioStore } from "../../src/helpers/studio/store";

export default function Preview() {
  const router = useRouter();
  const project = useProject();
  const root = useStudioStore((state) => state.rootHandle);

  useEffect(() => {
    if (!root) {
      router.push("/studio");
    }
  }, [root, router]);

  if (!project) return null;

  return (
    <div className="h-full">
      <Head>
        <title>{project.name} / The Wired </title>
      </Head>

      <div className="crosshair" />

      <Canvas className="w-full h-full">
        <InstancedScene scene={project.scene}>
          <Player />
        </InstancedScene>
      </Canvas>

      <div
        onClick={(e) => e.stopPropagation()}
        className="fixed top-0 right-0 p-6 text-2xl"
      >
        <Link href={"/studio"} passHref>
          <div
            className="cursor-pointer p-2 rounded-full bg-surface text-onSurface
                       backdrop-blur bg-opacity-60 hover:bg-opacity-100 transition active:bg-opacity-90"
          >
            <MdClose />
          </div>
        </Link>
      </div>
    </div>
  );
}
