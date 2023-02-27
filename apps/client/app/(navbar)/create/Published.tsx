import Link from "next/link";

import { env } from "../../../src/env/server.mjs";
import { fetchProjects } from "../../../src/server/helpers/fetchProjects";
import Card from "../../../src/ui/Card";

function cdnImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publication/${id}/image.jpg`;
}

export default async function Published() {
  const projects = await fetchProjects();
  const publishedProjects = projects.filter((p) => p.Publication?.spaceId);

  if (publishedProjects.length === 0) return null;

  const publishedImages = publishedProjects.map((p) =>
    p.publicationId ? cdnImageURL(p.publicationId) : p.image
  );

  return (
    <>
      <div className="pt-4 text-2xl font-bold">🌍 Published</div>

      <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
        {publishedProjects.map(({ id, name }, i) => (
          <Link key={id} href={`/project/${id}`} className="rounded-xl">
            <Card text={name} image={publishedImages[i]} sizes="333px" animateEnter />
          </Link>
        ))}
      </div>
    </>
  );
}
