import { Metadata } from "next";
import Image from "next/image";
import Link from "next/link";
import { notFound } from "next/navigation";

import { fetchProject } from "@/src/server/helpers/fetchProject";
import ButtonTabs, { TabContent } from "@/src/ui/ButtonTabs";
import { isFromCDN } from "@/src/utils/isFromCDN";

import About from "./About";
import Settings from "./Settings";

type Params = { id: string };

export async function generateMetadata({ params: { id } }: { params: Params }): Promise<Metadata> {
  const project = await fetchProject(id);

  if (!project) return {};

  return {
    title: project.title,
    description: project.description,
  };
}

interface Props {
  params: Params;
}

export default async function Project({ params }: Props) {
  const { id } = params;
  const project = await fetchProject(id);

  if (!project) notFound();

  return (
    <div className="flex justify-center">
      <div className="max-w-content mx-4 space-y-8 py-8">
        <div className="flex flex-col space-y-8 md:flex-row md:space-x-8 md:space-y-0">
          <div className="aspect-card h-full w-full rounded-3xl bg-neutral-200">
            <div className="relative h-full w-full object-cover">
              {project.image &&
                (isFromCDN(project.image) ? (
                  <Image
                    src={project.image}
                    priority
                    fill
                    sizes="(min-width: 768px) 60vw, 100vw"
                    alt=""
                    className="rounded-3xl object-cover"
                  />
                ) : (
                  <img
                    src={project.image}
                    alt=""
                    sizes="(min-width: 768px) 60vw, 100vw"
                    className="h-full w-full rounded-3xl object-cover"
                    crossOrigin="anonymous"
                  />
                ))}
            </div>
          </div>

          <div className="flex flex-col justify-between space-y-8 md:w-2/3">
            <div className="space-y-4">
              <div className="flex justify-center text-3xl font-black">{project?.title}</div>
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
            {/* @ts-expect-error Server Component */}
            <About params={params} />
          </TabContent>
          <TabContent value="Settings">
            {/* @ts-expect-error Server Component */}
            <Settings params={params} />
          </TabContent>
        </ButtonTabs>
      </div>
    </div>
  );
}
