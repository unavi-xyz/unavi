import { useRouter } from "next/router";
import { useState } from "react";
import { MdArrowBackIosNew } from "react-icons/md";
import { useSigner } from "wagmi";

import { trpc } from "../../../client/trpc";
import SignInButton from "../../../home/NavbarLayout/SignInButton";
import Button from "../../../ui/Button";
import Dialog from "../../../ui/Dialog";
import { useSave } from "../../hooks/useSave";
import { useEditorStore } from "../../store";
import AutoGrowInput from "../ui/AutoGrowInput";
import PlayButton from "./PlayButton";
import PublishPage from "./PublishPage";
import ToolButtons from "./ToolButtons";
import UpdatePage from "./UpdatePage";
import VisualsButton from "./VisualsButton";

export default function EditorNavbar() {
  const name = useEditorStore((state) => state.name);
  const isSaving = useEditorStore((state) => state.isSaving);
  const [openPublishDialog, setOpenPublishDialog] = useState(false);

  const { save, saveImage } = useSave();
  const { data: signer } = useSigner();
  const router = useRouter();
  const id = router.query.id as string;

  const { data: project } = trpc.project.get.useQuery({ id }, { enabled: id !== undefined });

  async function handleBack() {
    await save();
    router.push(`/project/${id}`);
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
          <PlayButton />
          <VisualsButton />
          {signer ? <Button onClick={handleOpenPublish}>Publish</Button> : <SignInButton />}
        </div>
      </div>
    </>
  );
}
