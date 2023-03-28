import { fetchProjects } from "../../../src/server/helpers/fetchProjects";
import { getServerSession } from "../../../src/server/helpers/getServerSession";
import Card from "../../../src/ui/Card";
import CardGrid from "../../../src/ui/CardGrid";
import { cdnURL, S3Path } from "../../../src/utils/s3Paths";

export default async function Published() {
  const session = await getServerSession();

  if (!session) return null;

  const projects = await fetchProjects();
  const publishedProjects = projects.filter((p) => p.Publication?.spaceId);

  if (publishedProjects.length === 0) return null;

  const publishedImages = publishedProjects.map((p) =>
    p.publicationId ? cdnURL(S3Path.publication(p.publicationId).image) : p.image
  );

  return (
    <>
      <div className="pt-4 text-2xl font-bold">🌍 Published</div>

      <CardGrid>
        {publishedProjects.map(({ id, name }, i) => (
          <Card key={id} href={`/project/${id}`} text={name} image={publishedImages[i]} />
        ))}
      </CardGrid>
    </>
  );
}
