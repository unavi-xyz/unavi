import { DropdownContent, DropdownMenu, DropdownTrigger } from "@/src/ui/DropdownMenu";

import ProjectList from "./ProjectList";
import TitleDisplay from "./TitleDisplay";

interface Props {
  projectId: string;
}

export default function TitleButton({ projectId }: Props) {
  return (
    <DropdownMenu>
      <DropdownTrigger>
        <TitleDisplay />
      </DropdownTrigger>

      <DropdownContent>
        <ProjectList projectId={projectId} />
      </DropdownContent>
    </DropdownMenu>
  );
}
