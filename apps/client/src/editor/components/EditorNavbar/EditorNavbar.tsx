import { useRouter } from "next/navigation";
import { useState } from "react";
import { MdArrowBackIosNew } from "react-icons/md";

import SignInButton from "@/app/(navbar)/SignInButton";
import { MAX_TITLE_LENGTH } from "@/app/api/projects/constants";
import { useEditorStore } from "@/app/editor/[id]/store";

import { useSession } from "../../../client/auth/useSession";
import { Project } from "../../../server/helpers/fetchProject";
import Button from "../../../ui/Button";
import DialogContent, { DialogRoot } from "../../../ui/Dialog";
import { useSave } from "../../hooks/useSave";
import AutoGrowInput from "../ui/AutoGrowInput";
import PlayButton from "./PlayButton";
import PublishPage from "./PublishPage";
import ToolButtons from "./ToolButtons";
import VisualsButton from "./VisualsButton";

interface Props {
  project: Project;
}

export default function EditorNavbar({ project }: Props) {
  const router = useRouter();

  const name = useEditorStore((state) => state.title);
  const isSaving = useEditorStore((state) => state.isSaving);
  const sceneLoaded = useEditorStore((state) => state.sceneLoaded);
  const [openPublishDialog, setOpenPublishDialog] = useState(false);

  const { save, saveImage } = useSave(project.id);
  const { status } = useSession();

  const isPublished = Boolean(project?.spaceId);

  async function handleBack() {
    // Exit play mode
    const { stopPlaying } = useEditorStore.getState();
    await stopPlaying();

    await save();
    router.push(`/project/${project.id}`);
  }

  async function handleSave() {
    // Exit play mode
    const { stopPlaying } = useEditorStore.getState();
    await stopPlaying();

    await save();
  }

  async function handleOpenPublish() {
    if (!sceneLoaded) return;

    // Exit play mode
    const { stopPlaying } = useEditorStore.getState();
    await stopPlaying();

    // Start saving image
    saveImage();

    setOpenPublishDialog(true);
  }

  return (
    <>
      <DialogRoot open={openPublishDialog} onOpenChange={setOpenPublishDialog}>
        <DialogContent
          open={openPublishDialog}
          title={isPublished ? "Update Space" : "Publish Space"}
        >
          <PublishPage project={project} />
        </DialogContent>
      </DialogRoot>

      <div className="group flex h-full items-center justify-between px-4 py-1">
        <div className="flex w-full items-center space-x-2 text-lg">
          <button
            onClick={handleBack}
            className="cursor-pointer p-1 text-neutral-500 transition hover:text-inherit"
          >
            <MdArrowBackIosNew />
          </button>

          <div className="flex w-96 items-center">
            {status === "authenticated" ? (
              <AutoGrowInput
                type="text"
                name="title"
                autoComplete="off"
                maxLength={MAX_TITLE_LENGTH}
                value={name}
                onChange={(e) => useEditorStore.setState({ title: e.target.value })}
              />
            ) : null}

            <div className="flex items-center pt-0.5 pl-2">
              {isSaving ? (
                <div className="text-sm text-neutral-500">Saving...</div>
              ) : sceneLoaded ? (
                <button
                  onClick={handleSave}
                  className={
                    "rounded-md px-2 py-0.5 text-sm text-neutral-500 opacity-0 transition hover:bg-neutral-200 hover:text-neutral-900 focus:opacity-100 active:bg-neutral-200 group-hover:opacity-100"
                  }
                >
                  Save
                </button>
              ) : null}
            </div>
          </div>
        </div>

        <ToolButtons />

        <div className="flex h-full w-full items-center justify-end space-x-2">
          <PlayButton />
          <VisualsButton />
          {status === "authenticated" ? (
            <Button disabled={!sceneLoaded} onClick={handleOpenPublish}>
              Publish
            </Button>
          ) : (
            <SignInButton />
          )}
        </div>
      </div>
    </>
  );
}
