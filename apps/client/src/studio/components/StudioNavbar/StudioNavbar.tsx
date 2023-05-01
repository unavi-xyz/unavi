import { Project } from "../../../server/helpers/fetchProject";
import BackButton from "./BackButton";
import PlayButton from "./PlayButton";
import PublishButton from "./PublishButton";
import SaveButton from "./SaveButton";
import TitleButton from "./TitleButton/TitleButton";
import ToolButtons from "./ToolButtons";
import VisualsButton from "./VisualsButton";

interface Props {
  project: Project;
}

export default function StudioNavbar({ project }: Props) {
  return (
    <div className="flex h-12 items-center justify-between border-b px-4 py-1">
      <div className="flex w-full items-center space-x-4">
        <BackButton projectId={project.id} />
        <TitleButton projectId={project.id} />
        <SaveButton projectId={project.id} />
      </div>

      <ToolButtons />

      <div className="flex h-full w-full items-center justify-end space-x-2">
        <PlayButton />
        <VisualsButton />
        <PublishButton project={project} />
      </div>
    </div>
  );
}
