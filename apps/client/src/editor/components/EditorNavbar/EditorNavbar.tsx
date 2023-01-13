import { useRouter } from "next/router";
import { useState } from "react";
import { BiMove } from "react-icons/bi";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import { HiCubeTransparent } from "react-icons/hi";
import { MdArrowBackIosNew, MdPreview, MdSync } from "react-icons/md";

import Button from "../../../ui/Button";
import Dialog from "../../../ui/Dialog";
import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
import { useSave } from "../../hooks/useSave";
import { useEditorStore } from "../../store";
import AutoGrowInput from "../ui/AutoGrowInput";
import PublishPage from "./PublishPage";
import ToolButton from "./ToolButton";
import UpdatePage from "./UpdatePage";

export default function EditorNavbar() {
  const router = useRouter();
  const id = router.query.id as string;

  const visuals = useEditorStore((state) => state.visuals);
  const name = useEditorStore((state) => state.name);
  const publicationId = useEditorStore((state) => state.publicationId);
  const isSaving = useEditorStore((state) => state.isSaving);

  const [openPublishDialog, setOpenPublishDialog] = useState(false);

  const { save, saveImage } = useSave();

  function handleToggleColliders() {
    useEditorStore.setState({ visuals: !visuals });

    // const { engine } = useEditorStore.getState();
    // engine?.renderThread.postMessage({
    //   subject: "show_visuals",
    //   data: {
    //     visible: !visuals,
    //   },
    // });
  }

  async function handleBack() {
    await save();
    router.push(`/project/${id}`);
  }

  function handlePreview() {
    const { engine } = useEditorStore.getState();
    if (!engine) return;

    if (engine.controls === "player") engine.controls = "orbit";
    else engine.controls = "player";
  }

  async function handleOpenPublish() {
    await saveImage();
    setOpenPublishDialog(true);
  }

  return (
    <>
      <Dialog open={openPublishDialog} onClose={() => setOpenPublishDialog(false)}>
        {publicationId ? (
          <UpdatePage onClose={() => setOpenPublishDialog(false)} />
        ) : (
          <PublishPage />
        )}
      </Dialog>

      <div className="flex h-full items-center justify-between px-4 py-2">
        <div className="flex w-full items-center space-x-2 text-lg">
          <div
            onClick={handleBack}
            className="cursor-pointer p-1 text-neutral-500 transition hover:text-inherit"
          >
            <MdArrowBackIosNew />
          </div>

          <div className="flex w-96 items-center">
            <AutoGrowInput
              type="text"
              value={name}
              onChange={(e) => useEditorStore.setState({ name: e.target.value })}
            />

            {isSaving && <div className="pl-2 pt-0.5 text-sm text-neutral-500">Saving...</div>}
          </div>
        </div>

        <div className="flex h-full w-full items-center justify-center space-x-2">
          <ToolButton tool="translate">
            <BiMove />
          </ToolButton>

          <ToolButton tool="rotate">
            <MdSync />
          </ToolButton>

          <ToolButton tool="scale">
            <CgArrowsExpandUpRight />
          </ToolButton>
        </div>

        <div className="flex h-full w-full items-center justify-end space-x-2">
          <div className="aspect-square h-full">
            <Tooltip text={`${visuals ? "Hide" : "Show"} Visuals`} placement="bottom">
              <IconButton selected={visuals} onClick={handleToggleColliders}>
                <HiCubeTransparent />
              </IconButton>
            </Tooltip>
          </div>

          <div className="aspect-square h-full">
            <Tooltip text="Preview" placement="bottom">
              <div className="h-full">
                <IconButton onClick={handlePreview}>
                  <MdPreview />
                </IconButton>
              </div>
            </Tooltip>
          </div>

          <Button variant="filled" onClick={handleOpenPublish}>
            Publish
          </Button>
        </div>
      </div>
    </>
  );
}
