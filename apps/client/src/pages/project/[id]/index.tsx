import { useRouter } from "next/router";

import { trpc } from "../../../client/trpc";
import { env } from "../../../env/client.mjs";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import ProjectLayout from "../../../home/layouts/ProjectLayout/ProjectLayout";

function publicationImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publications/${id}/image.jpg`;
}

export default function Project() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data: project } = trpc.project.get.useQuery({ id }, { enabled: id !== undefined });
  const { data: imageURL } = trpc.project.image.useQuery(
    { id },
    { enabled: id !== undefined && !project?.publicationId }
  );

  // If published, use the publication image
  const image = project?.publicationId ? publicationImageURL(project.publicationId) : imageURL;

  return (
    <ProjectLayout name={project?.name} image={image}>
      <div className="whitespace-pre-line">{project?.description}</div>
    </ProjectLayout>
  );
}

Project.getLayout = getNavbarLayout;
