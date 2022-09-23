import Image from "next/future/image";
import Link from "next/link";

import Button from "./base/Button";

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
    imageSide === "right" ? "flex-row-reverse" : "flex-row";

  const isExternalLink = buttonLink.startsWith("http");

  return (
    <div className="h-screen snap-center pb-20">
      <div className="flex h-full flex-col md:flex-row md:items-center">
        <div className="h-full w-full py-2 md:h-1/2 md:w-1/2 md:p-8 md:py-0">
          <div className="relative h-full w-full">
            <Image
              src={image}
              fill
              sizes="293px"
              alt="Wired-chan"
              className="select-none object-contain"
            />
          </div>
        </div>

        <div className="w-full space-y-3">
          <div className="text-onPrimaryContainer bg-primaryContainer w-fit rounded-xl px-3 py-2 text-5xl font-black md:px-4 md:text-6xl">
            {title}
          </div>

          <div className="text-3xl md:text-5xl">{subtitle}</div>

          <div className="text-outline ml-1 text-lg md:text-xl">{body}</div>

          <div className="w-full pt-4 text-lg md:text-xl">
            {isExternalLink ? (
              <a href={buttonLink} target="_blank" rel="noreferrer">
                <div className="w-full md:w-fit">
                  <Button variant="filled" rounded="large" fullWidth>
                    <div className="md:px-5 md:py-0.5">{buttonText}</div>
                  </Button>
                </div>
              </a>
            ) : (
              <Link href={buttonLink} passHref>
                <div className="w-full md:w-fit">
                  <Button variant="filled" rounded="large" fullWidth>
                    <div className="md:px-5 md:py-0.5">{buttonText}</div>
                  </Button>
                </div>
              </Link>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
