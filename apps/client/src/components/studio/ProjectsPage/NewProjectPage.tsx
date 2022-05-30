import { useRef, useState } from "react";
import { MdArrowBackIosNew, MdOutlineFolderOpen } from "react-icons/md";

import {
  PROJECT_FILE_NAME,
  STARTING_SCENE,
} from "../../../helpers/studio/constants";
import { useStudioStore } from "../../../helpers/studio/store";
import { Project } from "../../../helpers/studio/types";
import Button from "../../base/Button";
import TextField from "../../base/TextField";

interface Props {
  onBack: () => void;
}

export default function NewProjectsPage({ onBack }: Props) {
  const nameRef = useRef<HTMLInputElement>(null);

  const [directory, setDirectory] = useState<FileSystemDirectoryHandle>();

  async function handleSelectDirectory() {
    try {
      const directoryHandle = await window.showDirectoryPicker();
      if (!directoryHandle) return;

      setDirectory(directoryHandle);
    } catch (err) {
      console.error(err);
    }
  }

  async function handleCreate() {
    if (!directory) return;

    try {
      //create project file
      const fileHandle = await directory.getFileHandle(PROJECT_FILE_NAME, {
        create: true,
      });

      //write to file
      const name = nameRef.current?.value ?? directory.name;
      const project: Project = {
        name,
        description: "",
        scene: STARTING_SCENE,
      };

      const writableStream = await fileHandle.createWritable();
      await writableStream.write(JSON.stringify(project));
      await writableStream.close();

      //open project
      useStudioStore.setState({
        rootHandle: directory,
        selectedDirectory: directory,
      });
    } catch (err) {
      console.error(err);
    }
  }

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <div className="w-full grid grid-cols-6 items-center">
          <div>
            <button
              onClick={onBack}
              className="p-1 rounded-full text-outline hover:text-inherit transition"
            >
              <MdArrowBackIosNew className="text-xl" />
            </button>
          </div>

          <div className="w-full flex justify-center col-span-4">
            <h1 className="text-3xl flex justify-center">New Project</h1>
          </div>
        </div>

        <p className="text-lg flex justify-center"></p>
      </div>

      <TextField inputRef={nameRef} title="Name" defaultValue={"New Project"} />

      <div className="space-y-2">
        <div className="text-lg font-bold">Location</div>

        <div>Select where to store the project in your filesystem</div>

        <Button
          variant="tonal"
          squared
          fullWidth
          onClick={handleSelectDirectory}
        >
          <div className="flex items-center justify-center space-x-2 py-2">
            <MdOutlineFolderOpen className="text-lg" />
            <div>Select Folder</div>
          </div>
        </Button>

        {directory && (
          <div>
            <div className="text-lg font-bold">Selected Folder:</div>
            <div>{directory.name}</div>
          </div>
        )}
      </div>

      <div className="flex justify-end">
        <Button variant="filled" disabled={!directory} onClick={handleCreate}>
          Create
        </Button>
      </div>
    </div>
  );
}
