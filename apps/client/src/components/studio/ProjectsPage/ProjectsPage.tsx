import { useState } from "react";
import { MdAdd, MdOutlineFolderOpen } from "react-icons/md";

import { PROJECT_FILE_NAME } from "../../../helpers/studio/constants";
import { useStudioStore } from "../../../helpers/studio/store";
import Button from "../../base/Button";
import BrowserErrorPage from "./BrowserErrorPage";
import NewProjectsPage from "./NewProjectPage";

export default function ProjectsPage() {
  const [newProject, setNewProject] = useState(false);

  async function handleOpenProject() {
    try {
      //get directory
      const directoryHandle = await window.showDirectoryPicker();

      // test if project file exists
      await directoryHandle.getFileHandle(PROJECT_FILE_NAME);

      //open project
      useStudioStore.setState({
        rootHandle: directoryHandle,
        selectedDirectory: directoryHandle,
      });
    } catch (err) {
      console.error(err);
    }
  }

  //if file access API not supported
  if (window && !window.showDirectoryPicker) return <BrowserErrorPage />;

  if (newProject)
    return <NewProjectsPage onBack={() => setNewProject(false)} />;

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Project</h1>
        <p className="text-lg flex justify-center">
          Open an existing project or create a new one
        </p>
      </div>

      <div className="flex justify-center space-x-4">
        <Button variant="tonal" squared fullWidth onClick={handleOpenProject}>
          <div className="flex items-center justify-center space-x-2 py-2">
            <MdOutlineFolderOpen className="text-lg" />
            <div>Open Project</div>
          </div>
        </Button>

        <Button
          variant="tonal"
          squared
          fullWidth
          onClick={() => setNewProject(true)}
        >
          <div className="flex items-center justify-center space-x-2 py-2">
            <MdAdd className="text-lg" />
            <div>New Project</div>
          </div>
        </Button>
      </div>
    </div>
  );
}
