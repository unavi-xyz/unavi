import { NodeIO } from "@gltf-transform/core";
import { Packet } from "@gltf-transform/extensions";
import { Space } from "@wired-labs/gltf-extensions";
import { extensions } from "engine";
import { cache } from "react";
import { z } from "zod";

import createDecoderModule from "@/public/scripts/draco_decoder";
import { env } from "@/src/env.mjs";

export const readSpaceMetadata = cache(async (uri: string): Promise<SpaceMetadata | null> => {
  try {
    const res = await fetch(uri, { next: { revalidate: 60 } });
    const buffer = await res.arrayBuffer();
    const array = new Uint8Array(buffer);

    const io = new NodeIO()
      .registerExtensions(extensions)
      .registerDependencies({ "draco3d.decoder": await createDecoderModule() });

    const doc = await io.readBinary(array);

    const spaceExtension = doc.getRoot().getExtension<Space>(Space.EXTENSION_NAME);
    if (!spaceExtension) throw new Error("No space extension.");

    // Read space metadata
    const spaceHost = spaceExtension.getHost();
    const image = spaceExtension.getImage();

    const host =
      process.env.NODE_ENV === "development"
        ? "localhost:4000"
        : spaceHost || env.NEXT_PUBLIC_DEFAULT_HOST;

    // Read xmp metadata
    const xmp = doc.getRoot().getExtension<Packet>(Packet.EXTENSION_NAME);
    const parsedCreator = z.string().safeParse(xmp?.getProperty("dc:creator"));
    const parsedDescription = z.string().safeParse(xmp?.getProperty("dc:description"));
    const parsedTitle = z.string().safeParse(xmp?.getProperty("dc:title"));

    const creator = parsedCreator.success ? parsedCreator.data : "";
    const description = parsedDescription.success ? parsedDescription.data : "";
    const title = parsedTitle.success ? parsedTitle.data : "";

    return { creator, description, host, image, title, uri };
  } catch {
    return null;
  }
});

export type SpaceMetadata = {
  creator: string;
  description: string;
  host: string;
  image: string;
  title: string;
  uri: string;
};
