"use client";

import { useState } from "react";

import { Project } from "@/src/server/helpers/fetchProject";

import Button from "../../../ui/Button";
import DialogContent, { DialogRoot } from "../../../ui/Dialog";
import { useSave } from "../../hooks/useSave";
import { useStudio } from "../Studio";
import PublishPage from "./PublishPage";

interface Props {
  project: Project;
}

export default function PublishButton({ project }: Props) {
  const [openPublishDialog, setOpenPublishDialog] = useState(false);

  const { loaded, changeMode } = useStudio();
  const { saveImage } = useSave(project.id);

  const isPublished = Boolean(project?.spaceId);

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
        <DialogContent title={isPublished ? "Update Space" : "Publish Space"}>
          <PublishPage project={project} />
        </DialogContent>
      </DialogRoot>

      <Button disabled={!loaded} onClick={handleOpenPublish}>
        Publish
      </Button>
    </>
  );
}
