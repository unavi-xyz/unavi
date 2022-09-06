import Image from "next/future/image";
import Link from "next/link";
import { useRouter } from "next/router";

import { trpc } from "../../src/auth/trpc";
import { getNavbarLayout } from "../../src/home/layouts/NavbarLayout/NavbarLayout";
import MetaTags from "../../src/ui/MetaTags";
import Button from "../../src/ui/base/Button";

export default function Project() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data, isFetched } = trpc.useQuery(["project", { id }], {
    enabled: id !== undefined,
  });

  if (!isFetched || !data) return null;

  return (
    <>
      <MetaTags title={data.name || "Create"} />

      <div className="mx-4 h-full">
        <div className="max-w mx-auto py-8 w-full h-full space-y-8">
          <div className="flex flex-col md:flex-row space-y-8 md:space-y-0 md:space-x-8">
            <div className="w-full h-full rounded-3xl aspect-card bg-primaryContainer">
              <div className="relative object-cover w-full h-full">
                {data.image && (
                  <Image
                    src={data.image}
                    priority
                    fill
                    sizes="40vw"
                    alt="project preview"
                    className="rounded-3xl object-cover"
                  />
                )}
              </div>
            </div>

            <div className="md:w-2/3 min-w-fit flex flex-col justify-between space-y-8">
              <div className="font-black text-3xl flex justify-center">
                {data.name}
              </div>

              <Link href={`/studio/${id}`} passHref>
                <a>
                  <Button variant="filled" fullWidth>
                    <div className="py-2">Open Studio</div>
                  </Button>
                </a>
              </Link>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

Project.getLayout = getNavbarLayout;
