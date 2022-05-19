import { Canvas } from "@react-three/fiber";
import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import { MdClose } from "react-icons/md";

import { InstancedScene } from "@wired-xr/scene";

import Player from "../../../src/components/app/Player";
import { useLocalSpace } from "../../../src/helpers/indexeddb/LocalSpace/hooks/useLocalScene";

export default function Preview() {
  const router = useRouter();
  const id = router.query.id as string;

  const space = useLocalSpace(id);

  if (!space) return null;

  return (
    <div className="h-full">
      <Head>
        <title>{space.name ?? space.id} · The Wired </title>
      </Head>

      <div className="crosshair" />

      <Canvas className="w-full h-full">
        <InstancedScene scene={space.scene}>
          <Player />
        </InstancedScene>
      </Canvas>

      <div
        onClick={(e) => e.stopPropagation()}
        className="fixed top-0 right-0 p-6 text-2xl"
      >
        <Link href={`/studio/${id}`} passHref>
          <div
            className="cursor-pointer p-2 rounded-full bg-primaryContainer text-onPrimaryContainer
                       backdrop-blur bg-opacity-50 hover:bg-opacity-90 transition active:bg-opacity-75"
          >
            <MdClose />
          </div>
        </Link>
      </div>
    </div>
  );
}
