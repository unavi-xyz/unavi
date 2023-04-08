import { fetchProject } from "@/src/server/helpers/fetchProject";

export default async function About({ params: { id } }: { params: { id: string } }) {
  const project = await fetchProject(id);

  return (
    <div className="space-y-12">
      {project?.description && (
        <div>
          <div className="text-lg font-semibold">Description</div>

          <div className="whitespace-pre-line text-neutral-800">{project.description}</div>
        </div>
      )}
    </div>
  );
}
