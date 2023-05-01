"use client";

import { useSave } from "../../hooks/useSave";
import { useStudio } from "../Studio";

interface Props {
  projectId: string;
}

export default function SaveButton({ projectId }: Props) {
  const { loaded } = useStudio();
  const { saving, save } = useSave(projectId);

  return (
    <div className="flex items-center pt-0.5">
      {saving ? (
        <div className="text-sm text-neutral-500">Saving...</div>
      ) : loaded ? (
        <button
          onClick={save}
          className={
            "rounded-md px-2 py-0.5 text-sm text-neutral-500 transition hover:bg-neutral-200 hover:text-neutral-900 active:bg-neutral-200"
          }
        >
          Save
        </button>
      ) : null}
    </div>
  );
}
