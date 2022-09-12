import { useRouter } from "next/router";

import { trpc } from "../../../src/auth/trpc";
import { getNavbarLayout } from "../../../src/home/layouts/NavbarLayout/NavbarLayout";
import ProjectLayout from "../../../src/home/layouts/ProjectLayout/ProjectLayout";

export default function Project() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data, isFetched } = trpc.useQuery(["project", { id }], {
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
