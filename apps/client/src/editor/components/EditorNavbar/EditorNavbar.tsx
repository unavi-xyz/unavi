import { useRouter } from "next/router";
import { useState } from "react";
import { FaPlay, FaStop } from "react-icons/fa";
import { HiCubeTransparent } from "react-icons/hi";
import { MdArrowBackIosNew } from "react-icons/md";

import { trpc } from "../../../client/trpc";
import Button from "../../../ui/Button";
import Dialog from "../../../ui/Dialog";
import IconButton from "../../../ui/IconButton";
import Tooltip from "../../../ui/Tooltip";
import { useSave } from "../../hooks/useSave";
import { useEditorStore } from "../../store";
import AutoGrowInput from "../ui/AutoGrowInput";
import PublishPage from "./PublishPage";
import ToolButtons from "./ToolButtons";
import UpdatePage from "./UpdatePage";

export default function EditorNavbar() {
  const router = useRouter();
  const id = router.query.id as string;

  const visuals = useEditorStore((state) => state.visuals);
  const name = useEditorStore((state) => state.name);
  const isSaving = useEditorStore((state) => state.isSaving);

  const [openPublishDialog, setOpenPublishDialog] = useState(false);
  const [playing, setPlaying] = useState(false);

  const { data: project } = trpc.project.get.useQuery({ id }, { enabled: id !== undefined });

  const { save, saveImage } = useSave();

  function handleToggleColliders() {
    const { engine } = useEditorStore.getState();
    if (!engine) return;

    engine.visuals = !visuals;
    useEditorStore.setState({ visuals: !visuals });
  }

  async function handleBack() {
    await save();
    router.push(`/project/${id}`);
  }

  function handlePlay() {
    const { engine } = useEditorStore.getState();
    if (!engine) return;

    if (engine.controls === "player") {
      setPlaying(false);
      engine.controls = "orbit";
      engine.behavior.stop();
    } else {
      setPlaying(true);
      engine.controls = "player";
      useEditorStore.setState({ selectedId: null });
      engine.physics.send({ subject: "respawn", data: null });
      engine.behavior.start();
    }
  }

  async function handleOpenPublish() {
    await saveImage();
    setOpenPublishDialog(true);
  }

  const isPublished = project?.Publication?.spaceId;

  return (
    <>
      <Dialog
        open={openPublishDialog}
        onOpenChange={setOpenPublishDialog}
        title={isPublished ? "Update Space" : "Publish Space"}
      >
        {isPublished ? <UpdatePage onClose={() => setOpenPublishDialog(false)} /> : <PublishPage />}
      </Dialog>

      <div className="flex h-full items-center justify-between px-4 py-1">
        <div className="flex w-full items-center space-x-2 text-lg">
          <button
            onClick={handleBack}
            className="cursor-pointer p-1 text-neutral-500 transition hover:text-inherit"
          >
            <MdArrowBackIosNew />
          </button>

          <div className="flex w-96 items-center">
            <AutoGrowInput
              type="text"
              value={name}
              onChange={(e) => useEditorStore.setState({ name: e.target.value })}
            />

            {isSaving && <div className="pl-2 pt-0.5 text-sm text-neutral-500">Saving...</div>}
          </div>
        </div>

        <ToolButtons />

        <div className="flex h-full w-full items-center justify-end space-x-2">
          <div className="aspect-square h-full">
            <Tooltip text={`${playing ? "Stop" : "Play"}`} side="bottom">
              <IconButton onClick={handlePlay}>
                {playing ? <FaStop className="text-sm" /> : <FaPlay className="text-sm" />}
              </IconButton>
            </Tooltip>
          </div>

          <div className="aspect-square h-full">
            <Tooltip text={`${visuals ? "Hide" : "Show"} Visuals`} side="bottom">
              <IconButton selected={visuals} onClick={handleToggleColliders}>
                <HiCubeTransparent />
              </IconButton>
            </Tooltip>
          </div>

          <Button onClick={handleOpenPublish}>Publish</Button>
        </div>
      </div>
    </>
  );
}
