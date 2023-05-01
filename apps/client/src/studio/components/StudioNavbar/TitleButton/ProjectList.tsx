import { fetchProjects } from "@/src/server/helpers/fetchProjects";

import CreateProjectButton from "./CreateProjectButton";
import ProjectItem from "./ProjectItem";
import ProjectMoreButton from "./ProjectMoreButton";

interface Props {
  projectId: string;
}

export default async function ProjectList({ projectId }: Props) {
  let projects = await fetchProjects();

  // Sort projects by last updated (put current project first)
  projects = projects.sort((a, b) => {
    if (a.publicId === projectId) return -1;
    if (b.publicId === projectId) return 1;
    return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
  });

  return (
    <div className="w-[420px] space-y-4 py-4">
      <div className="px-6 text-lg font-semibold">My Projects ({projects.length})</div>

      <div className="h-64 space-y-2 overflow-y-auto">
        {projects.map((project) => (
          <ProjectItem
            key={project.publicId}
            projectId={project.publicId}
            currentProject={project.publicId === projectId}
          >
            <div className="group flex w-full cursor-pointer items-center space-x-4">
              <div className="rounded-lg bg-neutral-200">
                {project.image ? (
                  <img
                    src={project.image}
                    alt="Project Image"
                    className="h-14 w-24 rounded-lg object-cover"
                    crossOrigin="anonymous"
                  />
                ) : null}
              </div>

              <div className="group-hover:opacity-70">
                <div className="font-semibold">{project.title || "Untitled"}</div>
                <div className="w-16 overflow-hidden text-ellipsis whitespace-nowrap text-sm leading-4 text-neutral-500">
                  {project.publicId}
                </div>
              </div>
            </div>

            <ProjectMoreButton
              projectId={project.publicId}
              currentProject={project.publicId === projectId}
            />
          </ProjectItem>
        ))}
      </div>

      <CreateProjectButton projectId={projectId} />
    </div>
  );
}
