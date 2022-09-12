import Image from "next/future/image";
import Link from "next/link";
import { useRouter } from "next/router";

import MetaTags from "../../../ui/MetaTags";
import Button from "../../../ui/base/Button";
import NavigationTab from "../../../ui/base/NavigationTab";

interface Props {
  name: string | null;
  image: string | null;
  children: React.ReactNode;
}

export default function ProjectLayout({ name, image, children }: Props) {
  const router = useRouter();
  const id = router.query.id as string;

  return (
    <>
      <MetaTags title={name || "Project"} />

      <div className="mx-4 h-full">
        <div className="max-w-content mx-auto py-8 w-full h-full space-y-8">
          <div className="flex flex-col md:flex-row space-y-8 md:space-y-0 md:space-x-8">
            <div className="w-full h-full rounded-3xl aspect-card bg-primaryContainer">
              <div className="relative object-cover w-full h-full">
                {image && (
                  <Image
                    src={image}
                    priority
                    fill
                    sizes="40vw"
                    alt="space preview"
                    className="rounded-3xl object-cover"
                  />
                )}
              </div>
            </div>

            <div className="md:w-2/3 min-w-fit flex flex-col justify-between space-y-8">
              <div className="space-y-4">
                <div className="font-black text-3xl flex justify-center">
                  {name}
                </div>
              </div>

              <Link href={`/editor/${id}`} passHref>
                <a>
                  <Button variant="filled" fullWidth>
                    <div className="py-2">Open Editor</div>
                  </Button>
                </a>
              </Link>
            </div>
          </div>

          <div className="space-y-4 pb-4">
            <div className="flex space-x-4">
              <NavigationTab href={`/project/${id}`} text="About" />
              <NavigationTab href={`/project/${id}/settings`} text="Settings" />
            </div>

            <div>{children}</div>
          </div>
        </div>
      </div>
    </>
  );
}
