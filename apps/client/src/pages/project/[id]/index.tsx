import { useRouter } from "next/router";

import { trpc } from "../../../auth/trpc";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import ProjectLayout from "../../../home/layouts/ProjectLayout/ProjectLayout";

export default function Project() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data, isFetched } = trpc.useQuery(["auth.project", { id }], {
    enabled: id !== undefined,
  });

  if (!isFetched || !data) return null;

  return (
    <ProjectLayout name={data.name} image={data.image}>
      <div>{data.description}</div>
    </ProjectLayout>
  );
}

Project.getLayout = getNavbarLayout;
