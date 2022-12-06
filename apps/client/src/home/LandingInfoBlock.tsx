import Image from "next/image";
import Link from "next/link";

import Button from "../ui/Button";

interface Props {
  title: string;
  subtitle: string;
  body: string;
  buttonText: string;
  buttonLink: string;
  image: string;
  imageSide: "left" | "right";
}

export default function LandingInfoBlock({
  title,
  subtitle,
  body,
  buttonText,
  buttonLink,
  image,
  imageSide,
}: Props) {
  const directionClass =
    imageSide === "right" ? "md:flex-row-reverse" : "md:flex-row";

  const isExternalLink = buttonLink.startsWith("http");

  return (
    <div className="h-screen snap-center py-20">
      <div className={`flex h-full flex-col md:items-center ${directionClass}`}>
        <div className="h-full w-full py-2 md:h-1/2 md:w-1/2 md:p-8 md:py-0">
          <div className="relative h-full w-full">
            <Image
              src={image}
              fill
              loading="eager"
              sizes="293px"
              alt="Wired-chan"
              className="select-none object-contain"
            />
          </div>
        </div>

        <div className="w-full space-y-3">
          <div className="w-fit rounded-xl bg-primaryContainer px-3 py-2 text-5xl font-black text-onPrimaryContainer md:px-4 md:text-6xl">
            {title}
          </div>

          <div className="text-3xl md:text-5xl">{subtitle}</div>

          <div className="ml-1 text-lg text-outline md:text-xl">{body}</div>

          <div className="w-full pt-4 text-lg md:text-xl">
            {isExternalLink ? (
              <div className="w-full md:w-fit">
                <a href={buttonLink} target="_blank" rel="noreferrer">
                  <Button variant="filled" rounded="large" fullWidth>
                    <div className="md:px-3 md:py-0.5">{buttonText}</div>
                  </Button>
                </a>
              </div>
            ) : (
              <div className="w-full md:w-fit">
                <Link href={buttonLink}>
                  <Button variant="filled" rounded="large" fullWidth>
                    <div className="md:px-3 md:py-0.5">{buttonText}</div>
                  </Button>
                </Link>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
