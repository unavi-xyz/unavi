import { useRouter } from "next/navigation";
import { useState } from "react";
import { MdArrowBackIosNew } from "react-icons/md";

import SignInButton from "@/app/(navbar)/SignInButton";
import { MAX_TITLE_LENGTH } from "@/app/api/projects/constants";
import { useAuth } from "@/src/client/AuthProvider";

import { Project } from "../../../server/helpers/fetchProject";
import Button from "../../../ui/Button";
import DialogContent, { DialogRoot } from "../../../ui/Dialog";
import { useSave } from "../../hooks/useSave";
import { useEditor } from "../Editor";
import AutoGrowInput from "../ui/AutoGrowInput";
import PlayButton from "./PlayButton";
import PublishPage from "./PublishPage";
import ToolButtons from "./ToolButtons";
import VisualsButton from "./VisualsButton";

interface Props {
  project: Project;
}

export default function EditorNavbar({ project }: Props) {
  const [openPublishDialog, setOpenPublishDialog] = useState(false);

  const { title, setTitle, loaded, changeMode } = useEditor();
  const { saving, save, saveImage } = useSave(project);
  const { status } = useAuth();
  const router = useRouter();

  const isPublished = Boolean(project?.spaceId);

  async function handleBack() {
    await save();
    router.push(`/project/${project.id}`);
  }

  async function handleOpenPublish() {
    if (!loaded) return;

    // Exit play mode
    await changeMode("edit");

    // Start saving image
    saveImage();

    // Wait for image to be saved
    await new Promise((resolve) => setTimeout(resolve, 30));

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
                value={title}
                onChange={(e) => setTitle(e.target.value)}
              />
            ) : null}

            <div className="flex items-center pl-2 pt-0.5">
              {saving ? (
                <div className="text-sm text-neutral-500">Saving...</div>
              ) : loaded ? (
                <button
                  onClick={save}
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
            <Button disabled={!loaded} onClick={handleOpenPublish}>
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
