import { fetchProject } from "../../../../../src/server/helpers/fetchProject";
import Connect from "./Connect";
import Delete from "./Delete";
import Download from "./Download";

export default async function Settings({ params: { id } }: { params: { id: string } }) {
  const project = await fetchProject(id);

  return (
    <div className="space-y-12">
      <Download id={id} projectName={project.name} />
      <Connect id={id} connectedSpaceId={project.Publication?.spaceId ?? undefined} />
      <Delete id={id} />
    </div>
  );
}
