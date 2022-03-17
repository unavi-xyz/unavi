import { useEffect } from "react";
import { IoMdClose } from "react-icons/io";
import { useRouter } from "next/router";
import Head from "next/head";
import { useAtom, useSetAtom } from "jotai";

import { previewModeAtom, worldIdAtom } from "../../../helpers/editor/state";
import Navbar from "../../../components/editor/Navbar/Navbar";
import EditorSidebar from "../../../components/editor/EditorSidebar/EditorSidebar";
import EditorCanvas from "../../../components/editor/EditorCanvas/EditorCanvas";
import PreviewCanvas from "../../../components/editor/EditorCanvas/PreviewCanvas";
import { useStore } from "../../../helpers/editor/store";

export default function Edit() {
  const router = useRouter();
  const id = router.query.id as string;

  const setScene = useStore((state) => state.setScene);
  const setSelected = useStore((state) => state.setSelected);

  const setWorldId = useSetAtom(worldIdAtom);
  const [previewMode, setPreviewMode] = useAtom(previewModeAtom);

  useEffect(() => {
    setWorldId(id);

    return () => {
      setScene({ instances: {}, textures: {} });
      setSelected(null);
      setWorldId(null);
    };
  }, [id, setScene, setSelected, setWorldId]);

  return (
    <>
      <Head>
        <title>The Wired - Editor</title>
      </Head>

      {previewMode ? (
        <div className="relative h-full">
          <PreviewCanvas />

          <div className="crosshair" />

          <div
            onClick={() => setPreviewMode(false)}
            className="absolute top-6 right-6 hover:cursor-pointer bg-black rounded-full p-1.5
                       bg-opacity-30 hover:bg-opacity-40 backdrop-blur-sm"
          >
            <IoMdClose className="text-3xl text-white" />
          </div>
        </div>
      ) : (
        <div className="h-full overflow-hidden">
          <Navbar id={id} />

          <div className="h-full bg-neutral-200 flex">
            <EditorCanvas />

            <div className="h-full w-full max-w-md">
              <EditorSidebar />
            </div>
          </div>
        </div>
      )}
    </>
  );
}
