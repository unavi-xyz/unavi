import { MdAdd } from "react-icons/md";

import { getSession } from "@/src/server/auth/getSession";
import DialogContent, { DialogRoot, DialogTrigger } from "@/src/ui/Dialog";

import CreateProjectPage from "./CreateProjectPage";

export default async function CreateProjectButton() {
  const session = await getSession();

  return (
    <DialogRoot>
      <DialogContent title="Create Project">
        <CreateProjectPage />
      </DialogContent>

      <DialogTrigger asChild>
        <button
          disabled={!session}
          className={`rounded-lg px-5 py-1.5 ring-1 ring-neutral-700 transition ${
            session ? "hover:bg-neutral-200 active:opacity-80" : "cursor-default opacity-40"
          }`}
        >
          <MdAdd className="text-lg" />
        </button>
      </DialogTrigger>
    </DialogRoot>
  );
}
