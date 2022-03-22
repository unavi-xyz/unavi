import { useEffect } from "react";
import { IoMdClose } from "react-icons/io";
import { useRouter } from "next/router";
import Head from "next/head";

import {
  editorManager,
  useStore,
} from "../../../components/editor/helpers/store";

import EditorNavbar from "../../../components/editor/EditorNavbar/EditorNavbar";
import EditorSidebar from "../../../components/editor/EditorSidebar/EditorSidebar";
import EditorCanvas from "../../../components/editor/EditorCanvas/EditorCanvas";
import PreviewCanvas from "../../../components/editor/EditorCanvas/PreviewCanvas";

export default function Edit() {
  const router = useRouter();
  const id = router.query.id as string;

  const previewMode = useStore((state) => state.previewMode);

  useEffect(() => {
    editorManager.setSceneId(id);

    return () => {
      editorManager.setSceneId(undefined);
    };
  }, [id]);

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
            onClick={() => editorManager.setPreviewMode(false)}
            className="absolute top-6 right-6 hover:cursor-pointer bg-black rounded-full p-1.5
                       bg-opacity-30 hover:bg-opacity-40 backdrop-blur-sm"
          >
            <IoMdClose className="text-3xl text-white" />
          </div>
        </div>
      ) : (
        <div className="h-full overflow-hidden">
          <EditorNavbar id={id} />

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
