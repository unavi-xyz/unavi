"use client";

import { useRouter } from "next/navigation";
import { IoIosCheckmark, IoIosMore } from "react-icons/io";

import { deleteProject } from "@/app/api/projects/[id]/helper";
import {
  DropdownContent,
  DropdownItem,
  DropdownMenu,
  DropdownTrigger,
} from "@/src/ui/DropdownMenu";
import Tooltip from "@/src/ui/Tooltip";

interface Props {
  projectId: string;
  currentProject?: boolean;
}

export default function ProjectMoreButton({ projectId, currentProject = false }: Props) {
  const router = useRouter();

  return (
    <div
      onClick={(e) => {
        // Prevent projects dropdown from closing
        // Gives us a buffer around the button
        e.stopPropagation();
      }}
      className="flex items-center space-x-2 pl-4"
    >
      {currentProject && (
        <Tooltip text="Current Project" side="top">
          <div className="flex h-6 w-6 items-center justify-center rounded-full bg-neutral-900 text-white">
            <IoIosCheckmark className="text-xl" />
          </div>
        </Tooltip>
      )}

      <div className="flex flex-col justify-center">
        <DropdownMenu>
          <DropdownTrigger>
            <div className="flex h-6 w-6 items-center justify-center rounded-full bg-neutral-200 hover:bg-neutral-300">
              <IoIosMore />
            </div>
          </DropdownTrigger>

          <DropdownContent>
            <div className="py-2">
              <DropdownItem
                onClick={async () => {
                  // Delete project
                  await deleteProject(projectId);

                  // Refresh page
                  router.refresh();
                }}
                className="cursor-pointer px-4 focus:bg-neutral-200 focus:outline-0"
              >
                Delete Project
              </DropdownItem>
            </div>
          </DropdownContent>
        </DropdownMenu>
      </div>
    </div>
  );
}
