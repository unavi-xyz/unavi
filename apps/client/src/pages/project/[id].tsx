import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/router";

import { trpc } from "../../client/trpc";
import { env } from "../../env/client.mjs";
import MetaTags from "../../home/MetaTags";
import { getNavbarLayout } from "../../home/NavbarLayout/NavbarLayout";
import ProjectSettings from "../../home/ProjectSettings";
import ButtonTabs, { TabContent } from "../../ui/ButtonTabs";
import { isFromCDN } from "../../utils/isFromCDN";

function publicationImageURL(id: string) {
  return `https://${env.NEXT_PUBLIC_CDN_ENDPOINT}/publication/${id}/image.jpg`;
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
    <>
      <MetaTags title={project?.name || "Project"} />

      <div className="mx-4 h-full">
        <div className="max-w-content mx-auto h-full w-full space-y-8 py-8">
          <div className="flex flex-col space-y-8 md:flex-row md:space-y-0 md:space-x-8">
            <div className="aspect-card h-full w-full rounded-2xl bg-neutral-200">
              <div className="relative h-full w-full object-cover">
                {image &&
                  (isFromCDN(image) ? (
                    <Image
                      src={image}
                      priority
                      fill
                      sizes="40vw"
                      alt=""
                      className="rounded-2xl object-cover"
                    />
                  ) : (
                    <img
                      src={image}
                      alt=""
                      className="h-full w-full rounded-2xl object-cover"
                      crossOrigin="anonymous"
                    />
                  ))}
              </div>
            </div>

            <div className="flex flex-col justify-between space-y-8 md:w-2/3">
              <div className="space-y-4">
                <div className="flex justify-center text-3xl font-black">{project?.name}</div>
              </div>

              <Link
                href={`/editor/${id}`}
                className="rounded-full bg-neutral-900 py-3 text-center text-lg font-bold text-white transition hover:scale-105 active:opacity-90"
              >
                Open Editor
              </Link>
            </div>
          </div>

          <ButtonTabs titles={["About", "Settings"]}>
            <TabContent value="About">
              <div className="whitespace-pre-line">{project?.description}</div>
            </TabContent>
            <TabContent value="Settings">
              <ProjectSettings />
            </TabContent>
          </ButtonTabs>
        </div>
      </div>
    </>
  );
}

Project.getLayout = getNavbarLayout;
