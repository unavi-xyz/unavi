import { OrbitControls } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import Head from "next/head";
import Link from "next/link";
import { useRouter } from "next/router";
import { MdClose } from "react-icons/md";
import { InstancedScene } from "scene";
import { useLocalSpace } from "../../../src/helpers/indexedDB/localSpaces/hooks/useLocalScene";

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

      <Canvas className="w-full h-full">
        <InstancedScene scene={space.scene} />
        <OrbitControls />
      </Canvas>

      <div className="fixed top-0 right-0 p-6 text-2xl">
        <Link href={`/studio/${id}`} passHref>
          <div
            className="cursor-pointer p-2 rounded-full bg-black
                       backdrop-blur bg-opacity-10 hover:bg-opacity-20
                       transition-all duration-150"
          >
            <MdClose />
          </div>
        </Link>
      </div>
    </div>
  );
}
