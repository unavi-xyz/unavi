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
import PublishPage from "./PublishPage";
import ToolButton from "./ToolButton";

export default function EditorNavbar() {
  const router = useRouter();
  const id = router.query.id;

  const colliders = useEditorStore((state) => state.colliders);
  const name = useEditorStore((state) => state.name);

  const [openPublishDialog, setOpenPublishDialog] = useState(false);

  const { save } = useSave();

  function handleToggleColliders() {
    useEditorStore.setState({ colliders: !colliders });

    const { engine } = useEditorStore.getState();
    engine?.renderThread.postMessage({
      subject: "show_visuals",
      data: {
        visible: !colliders,
      },
    });
  }

  async function handleBack() {
    await save();
    router.push(`/project/${id}`);
  }

  async function handlePreview() {
    const { engine } = useEditorStore.getState();
    if (!engine) throw new Error("Engine not found");

    // Save scene
    await save();

    // Export scene to glTF
    const exportedScene = await engine.export();
    useEditorStore.setState({ exportedScene });

    // Navigate to preview page
    router.push(`/editor/${id}/preview`);
  }

  async function handleOpenPublish() {
    await save();
    setOpenPublishDialog(true);
  }

  return (
    <>
      <Dialog
        open={openPublishDialog}
        onClose={() => setOpenPublishDialog(false)}
      >
        <PublishPage />
      </Dialog>

      <div className="flex h-full items-center justify-between px-4 py-2">
        <div className="flex w-full items-center space-x-4 text-lg">
          <div
            onClick={handleBack}
            className="cursor-pointer p-1 text-outline transition hover:text-inherit"
          >
            <MdArrowBackIosNew />
          </div>

          <div>{name}</div>
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
            <Tooltip
              text={`${colliders ? "Hide" : "Show"} Colliders`}
              placement="bottom"
            >
              <IconButton selected={colliders} onClick={handleToggleColliders}>
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
