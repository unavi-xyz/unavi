import { fetchProject } from "@/src/server/helpers/fetchProject";

export default async function About({ params: { id } }: { params: { id: string } }) {
  const project = await fetchProject(id);

  return <div className="whitespace-pre-line">{project?.description}</div>;
}
