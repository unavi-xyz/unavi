import { getSession } from "@/src/server/auth/getSession";
import { fetchProjects } from "@/src/server/helpers/fetchProjects";
import Card from "@/src/ui/Card";
import CardGrid from "@/src/ui/CardGrid";

export default async function Projects() {
  const session = await getSession();

  if (!session) {
    return <div className="text-neutral-500">You must be signed in to create a project.</div>;
  }

  const projects = await fetchProjects();
  const unpublishedProjects = projects.filter((p) => !p.spaceId);

  if (unpublishedProjects.length === 0) {
    return <div className="text-neutral-500">No projects found.</div>;
  }

  return (
    <CardGrid>
      {unpublishedProjects.map(({ publicId, title, image }) => (
        <Card key={publicId} href={`/project/${publicId}`} text={title} image={image} />
      ))}
    </CardGrid>
  );
}
