import Link from "next/link";

import SidebarButton from "./SidebarButton";

interface Props {
  children: React.ReactNode;
}

export default function Sidebar({ children }: Props) {
  return (
    <div className="flex h-full">
      <div className="w-96 flex-shrink-0 px-8 py-6 h-full bg-white space-y-4 z-40 overflow-y-scroll">
        <Link href="/">
          <div className="text-3xl font-bold cursor-pointer text-center">Examples</div>
        </Link>

        <div className="text-xl font-bold">Showcase</div>

        <div className="space-y-2">
          <SidebarButton href="/example/AntiqueCamera">Antique Camera</SidebarButton>
          <SidebarButton href="/example/Avocado">Avocado</SidebarButton>
          <SidebarButton href="/example/BarramundiFish">Barramundi Fish</SidebarButton>
          <SidebarButton href="/example/BoomBox">Boom Box</SidebarButton>
          <SidebarButton href="/example/Corset">Corset</SidebarButton>
          <SidebarButton href="/example/DamagedHelmet">Damaged Helmet</SidebarButton>
          <SidebarButton href="/example/FlightHelmet">Flight Helmet</SidebarButton>
          <SidebarButton href="/example/Lantern">Lantern</SidebarButton>
          <SidebarButton href="/example/SciFiHelmet">SciFi Helmet</SidebarButton>
          <SidebarButton href="/example/Suzanne">Suzanne</SidebarButton>
          <SidebarButton href="/example/WaterBottle">Water Bottle</SidebarButton>
        </div>

        <div className="text-xl font-bold">Feature Tests</div>

        <div className="space-y-2">
          <SidebarButton href="/example/AlphaBlendModeTest">Alpha Blend Mode Test</SidebarButton>
          <SidebarButton href="/example/BoomBoxWithAxes">Boom Box With Axes</SidebarButton>
          <SidebarButton href="/example/MetalRoughSpheres">Metal Rough Spheres</SidebarButton>
          <SidebarButton href="/example/MetalRoughSpheresNoTextures">
            Metal Rough Spheres No Textures
          </SidebarButton>
          <SidebarButton href="/example/MorphPrimitivesTest">Morph Primitives Test</SidebarButton>
          <SidebarButton href="/example/MorphStressTest">Morph Stress Test</SidebarButton>
          <SidebarButton href="/example/MultiUVTest">Multi UV Test</SidebarButton>
          <SidebarButton href="/example/NormalTangentTest">Normal Tangent Test</SidebarButton>
          <SidebarButton href="/example/NormalTangentMirrorTest">
            Normal Tangent Mirror Test
          </SidebarButton>
          <SidebarButton href="/example/OrientationTest">Orientation Test</SidebarButton>
          <SidebarButton href="/example/RecursiveSkeletons">Recursive Skeletons</SidebarButton>
          <SidebarButton href="/example/TextureCoordinateTest">
            Texture Coordinate Test
          </SidebarButton>
          <SidebarButton href="/example/TextureLinearInterpolationTest">
            Texture Linear Interpolation Test
          </SidebarButton>
          <SidebarButton href="/example/TextureSettingsTest">Texture Settings Test</SidebarButton>
          <SidebarButton href="/example/VertexColorTest">Vertex Color Test</SidebarButton>
        </div>

        <div className="text-xl font-bold">Minimal Tests</div>

        <div className="space-y-2">
          <SidebarButton href="/example/TriangleWithoutIndices">
            Triangle Without Indices
          </SidebarButton>
          <SidebarButton href="/example/Triangle">Triangle</SidebarButton>
          <SidebarButton href="/example/AnimatedTriangle">Animated Triangle</SidebarButton>
          <SidebarButton href="/example/AnimatedMorphCube">Animated Morph Cube</SidebarButton>
          <SidebarButton href="/example/AnimatedMorphSphere">Animated Morph Sphere</SidebarButton>
          <SidebarButton href="/example/SimpleMeshes">Simple Meshes</SidebarButton>
          <SidebarButton href="/example/SimpleMorph">Simple Morph</SidebarButton>
          <SidebarButton href="/example/SimpleSparseAccessor">Simple Sparse Accessor</SidebarButton>
          <SidebarButton href="/example/SimpleSkin">Simple Skin</SidebarButton>
          <SidebarButton href="/example/InterpolationTest">Interpolation Test</SidebarButton>
          <SidebarButton href="/example/UnicodeTest">Unicode Test</SidebarButton>
        </div>
      </div>

      <div className="w-full">{children}</div>
    </div>
  );
}
