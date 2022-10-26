import { useRouter } from "next/router";

import { trpc } from "../../../client/trpc";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import ProjectLayout from "../../../home/layouts/ProjectLayout/ProjectLayout";

export default function Project() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data: project } = trpc.auth.project.useQuery(
    { id },
    {
      enabled: id !== undefined,
      trpc: {},
    }
  );

  const { data: imageURL } = trpc.auth.projectImage.useQuery(
    { id },
    {
      enabled: id !== undefined,
      trpc: {},
    }
  );

  return (
    <ProjectLayout name={project?.name} image={imageURL}>
      <div>{project?.description}</div>
    </ProjectLayout>
  );
}

Project.getLayout = getNavbarLayout;
