import Link from "next/link";

import { fetchProjects } from "../../../src/server/helpers/fetchProjects";
import Card from "../../../src/ui/Card";

export default async function Projects() {
  const projects = await fetchProjects();
  const unpublishedProjects = projects.filter((p) => !p.Publication?.spaceId);

  return unpublishedProjects.map(({ id, name, image }) => (
    <Link key={id} href={`/project/${id}`} className="rounded-xl">
      <Card text={name} image={image} sizes="333px" animateEnter />
    </Link>
  ));
}
