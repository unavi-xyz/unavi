import { Canvas } from "@react-three/fiber";
import produce from "immer";
import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import { useEffect, useState } from "react";
import { MdClose } from "react-icons/md";

import { InstancedScene } from "@wired-xr/scene";

import Player from "../../src/components/app/Player";
import { readFileByPath } from "../../src/helpers/studio/filesystem";
import { useProject } from "../../src/helpers/studio/hooks/useProject";
import { useStudioStore } from "../../src/helpers/studio/store";
import { Project } from "../../src/helpers/studio/types";

export default function Preview() {
  const router = useRouter();
  const project = useProject();
  const root = useStudioStore((state) => state.rootHandle);

  const [loadedProject, setLoadedProject] = useState<Project>();

  useEffect(() => {
    if (!root) {
      router.push("/studio");
    }
  }, [root, router]);

  useEffect(() => {
    async function loadProject() {
      if (!project) {
        setLoadedProject(undefined);
        return;
      }

      const newProject = await produce(project, async (draft) => {
        //fetch assets
        await Promise.all(
          Object.entries(draft.scene.assets).map(async ([id, asset]) => {
            const file = await readFileByPath(asset.uri);
            if (!file) return;

            const url = URL.createObjectURL(file);
            draft.scene.assets[id].data = url;
          })
        );
      });

      setLoadedProject(newProject);
    }

    loadProject();
  }, [project]);

  if (!loadedProject) return null;

  return (
    <div className="h-full">
      <Head>
        <title>{loadedProject.name} / The Wired </title>
      </Head>

      <div className="crosshair" />

      <Canvas className="w-full h-full">
        <InstancedScene scene={loadedProject.scene}>
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
