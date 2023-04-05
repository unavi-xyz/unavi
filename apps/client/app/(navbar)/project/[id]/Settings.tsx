import { notFound } from "next/navigation";

import { fetchProject } from "@/src/server/helpers/fetchProject";

import Connect from "./Connect";
import Delete from "./Delete";
import Download from "./Download";

export default async function Settings({ params: { id } }: { params: { id: string } }) {
  const project = await fetchProject(id);

  if (!project) notFound();

  return (
    <div className="space-y-12">
      <Download id={id} projectName={project.name} />
      <Connect
        id={id}
        owner={project.owner}
        connectedSpaceId={project.publication?.spaceId ?? undefined}
      />
      <Delete id={id} />
    </div>
  );
}
