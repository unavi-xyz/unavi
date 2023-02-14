import { useRouter } from "next/router";
import { useState } from "react";
import { BiMove } from "react-icons/bi";
import { CgArrowsExpandUpRight } from "react-icons/cg";
import { FaPlay, FaStop } from "react-icons/fa";
import { HiCubeTransparent } from "react-icons/hi";
import { MdArrowBackIosNew, MdSync } from "react-icons/md";

import { trpc } from "../../../client/trpc";
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
    } else {
      setPlaying(true);
      engine.controls = "player";
      useEditorStore.setState({ selectedId: null });
      engine.physics.send({ subject: "respawn", data: null });
    }
  }

  async function handleOpenPublish() {
    await saveImage();
    setOpenPublishDialog(true);
  }

  return (
    <>
      <Dialog open={openPublishDialog} onClose={() => setOpenPublishDialog(false)}>
        {project?.Publication?.spaceId ? (
          <UpdatePage onClose={() => setOpenPublishDialog(false)} />
        ) : (
          <PublishPage />
        )}
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
            <Tooltip text={`${playing ? "Stop" : "Play"}`} placement="bottom">
              <div className="h-full">
                <IconButton selected={playing} onClick={handlePlay}>
                  {playing ? <FaStop className="text-sm" /> : <FaPlay className="text-sm" />}
                </IconButton>
              </div>
            </Tooltip>
          </div>

          <div className="aspect-square h-full">
            <Tooltip text={`${visuals ? "Hide" : "Show"} Visuals`} placement="bottom">
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
