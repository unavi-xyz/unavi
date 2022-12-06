import Image from "next/image";
import Link from "next/link";

import { getNavbarLayout } from "../../home/layouts/NavbarLayout/NavbarLayout";

export default function Create() {
  return (
    <div className="mx-4 flex justify-center py-8">
      <div className="max-w-content space-y-8">
        <div className="flex justify-center text-3xl font-black">Create</div>

        <div className="flex flex-col justify-center space-y-4 md:flex-row md:space-y-0 md:space-x-4">
          <CreateButton
            title="Spaces"
            subtitle="Create 3D spaces with the visual editor"
            image="/images/Spaces.png"
            href="/create/spaces"
          />
          <CreateButton
            title="Avatars"
            subtitle="Upload VRM avatars for you or others to use"
            image="/images/Avatars.png"
            href="/create/avatars"
          />
        </div>
      </div>
    </div>
  );
}

Create.getLayout = getNavbarLayout;

interface Props {
  href: string;
  title?: string;
  subtitle?: string;
  image?: string;
}

function CreateButton({ href, title, subtitle, image }: Props) {
  return (
    <div className="group w-full">
      <Link href={href}>
        <div className="h-full w-full space-y-2 rounded-3xl border-4 border-background p-6 transition duration-300 hover:border-black">
          <div className="w-full text-center text-3xl font-bold">{title}</div>
          <div className="w-full text-center text-xl text-outline">
            {subtitle}
          </div>

          <div className="flex h-64 w-full items-center justify-center pt-2">
            <div className="relative h-full w-full overflow-hidden rounded-2xl">
              {image && (
                <Image
                  src={image}
                  alt=""
                  sizes="440px"
                  fill
                  className="object-cover transition duration-500 group-hover:scale-105"
                />
              )}
            </div>
          </div>
        </div>
      </Link>
    </div>
  );
}
