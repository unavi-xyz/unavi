"use client";

import { useRouter } from "next/navigation";

import { useSave } from "@/src/studio/hooks/useSave";
import { DropdownItem } from "@/src/ui/DropdownMenu";

interface Props {
  children: React.ReactNode;
  projectId: string;
  currentProject?: boolean;
}

export default function ProjectItem({ children, projectId, currentProject = false }: Props) {
  const { save } = useSave(projectId);
  const router = useRouter();

  async function handleClick() {
    // If current project, do nothing
    if (currentProject) return;

    // Save the current project
    await save();

    // Redirect to the new project
    router.push(`/studio/${projectId}`);
  }

  return (
    <DropdownItem
      onClick={handleClick}
      className="relative mx-6 flex items-stretch rounded-lg focus:outline-0"
    >
      {children}
    </DropdownItem>
  );
}
