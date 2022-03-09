import { useEffect } from "react";
import { useRouter } from "next/router";
import { useAtom } from "jotai";

import Navbar from "../../../components/editor/Navbar/Navbar";
import EditorSidebar from "../../../components/editor/EditorSidebar/EditorSidebar";
import EditorCanvas from "../../../components/editor/EditorCanvas/EditorCanvas";
import { worldIdAtom } from "../../../helpers/editor/state";
import Head from "next/head";

export default function Id() {
  const router = useRouter();
  const id = router.query.id as string;

  const [, setWorldId] = useAtom(worldIdAtom);

  useEffect(() => {
    setWorldId(id);
  }, [id, setWorldId]);

  return (
    <>
      <Head>
        <title>The Wired - Editor</title>
      </Head>

      <div className="h-full">
        <Navbar id={id} />

        <div className="h-full bg-neutral-200 flex">
          <EditorCanvas />

          <div className="h-full w-full max-w-md">
            <EditorSidebar />
          </div>
        </div>
      </div>
    </>
  );
}
