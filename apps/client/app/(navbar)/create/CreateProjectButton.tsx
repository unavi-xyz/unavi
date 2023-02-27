import { MdAdd } from "react-icons/md";

import { getServerSession } from "../../../src/server/helpers/getServerSession";
import DialogContent, { DialogRoot, DialogTrigger } from "../../../src/ui/Dialog";
import CreateProjectPage from "./CreateProjectPage";

export default async function CreateProjectButton() {
  const session = await getServerSession();

  return (
    <DialogRoot>
      <DialogContent title="Create Project">
        <CreateProjectPage />
      </DialogContent>

      <DialogTrigger asChild>
        <button
          disabled={!session}
          className={`rounded-lg px-5 py-1.5 ring-1 ring-neutral-700 transition ${
            session ? "hover:bg-neutral-200 active:opacity-80" : "cursor-not-allowed opacity-40"
          }`}
        >
          <MdAdd className="text-lg" />
        </button>
      </DialogTrigger>
    </DialogRoot>
  );
}
