import Link from "next/link";

import SidebarButton from "./SidebarButton";

interface Props {
  children: React.ReactNode;
}

export default function Sidebar({ children }: Props) {
  return (
    <div className="flex h-full">
      <div className="z-40 h-full w-96 shrink-0 space-y-4 overflow-y-scroll bg-white px-8 py-6">
        <Link href="/">
          <div className="cursor-pointer text-center text-3xl font-bold">
            Examples
          </div>
        </Link>

        <div className="text-xl font-bold">Showcase</div>

        <div className="space-y-2">
          <SidebarButton href="/example/AntiqueCamera">
            Antique Camera
          </SidebarButton>
          <SidebarButton href="/example/Avocado">Avocado</SidebarButton>
          <SidebarButton href="/example/BarramundiFish">
            Barramundi Fish
          </SidebarButton>
          <SidebarButton href="/example/BoomBox">Boom Box</SidebarButton>
          <SidebarButton href="/example/Corset">Corset</SidebarButton>
          <SidebarButton href="/example/DamagedHelmet">
            Damaged Helmet
          </SidebarButton>
          <SidebarButton href="/example/FlightHelmet">
            Flight Helmet
          </SidebarButton>
          <SidebarButton href="/example/Lantern">Lantern</SidebarButton>
          <SidebarButton href="/example/SciFiHelmet">
            SciFi Helmet
          </SidebarButton>
          <SidebarButton href="/example/Suzanne">Suzanne</SidebarButton>
          <SidebarButton href="/example/WaterBottle">
            Water Bottle
          </SidebarButton>
        </div>

        <div className="text-xl font-bold">Standard</div>

        <div className="space-y-2">
          <SidebarButton href="/example/Box">Box</SidebarButton>
          <SidebarButton href="/example/BoxInterleaved">
            Box Interleaved
          </SidebarButton>
          <SidebarButton href="/example/BoxTextured">
            Box Textured
          </SidebarButton>
          <SidebarButton href="/example/BoxTexturedNonPowerOfTwo">
            Box Textured Non Power Of Two
          </SidebarButton>
          <SidebarButton href="/example/BoxWithSpaces">
            Box With Spaces
          </SidebarButton>
          <SidebarButton href="/example/BoxVertexColors">
            Box Vertex Colors
          </SidebarButton>
          <SidebarButton href="/example/Cube">Cube</SidebarButton>
          <SidebarButton href="/example/AnimatedCube">
            Animated Cube
          </SidebarButton>
          <SidebarButton href="/example/Duck">Duck</SidebarButton>
          <SidebarButton href="/example/TwoCylinderEngine">
            2 Cylinder Engine
          </SidebarButton>
          <SidebarButton href="/example/ReciprocatingSaw">
            Reciprocating Saw
          </SidebarButton>
          <SidebarButton href="/example/GearboxAssy">
            Gearbox Assy
          </SidebarButton>
          <SidebarButton href="/example/Buggy">Buggy</SidebarButton>
          <SidebarButton href="/example/BoxAnimated">
            Box Animated
          </SidebarButton>
          <SidebarButton href="/example/CesiumMilkTruck">
            Cesium Milk Truck
          </SidebarButton>
          <SidebarButton href="/example/RiggedSimple">
            Rigged Simple
          </SidebarButton>
          <SidebarButton href="/example/RiggedFigure">
            Rigged Figure
          </SidebarButton>
          <SidebarButton href="/example/CesiumMan">Cesium Man</SidebarButton>
          <SidebarButton href="/example/BrainStem">Brain Stem</SidebarButton>
          <SidebarButton href="/example/Fox">Fox</SidebarButton>
          <SidebarButton href="/example/VirtualCity">
            Virtual City
          </SidebarButton>
          <SidebarButton href="/example/Sponza">Sponza</SidebarButton>
          <SidebarButton href="/example/TwoSidedPlane">
            Two Sided Plane
          </SidebarButton>
        </div>

        <div className="text-xl font-bold">Feature Tests</div>

        <div className="space-y-2">
          <SidebarButton href="/example/AlphaBlendModeTest">
            Alpha Blend Mode Test
          </SidebarButton>
          <SidebarButton href="/example/BoomBoxWithAxes">
            Boom Box With Axes
          </SidebarButton>
          <SidebarButton href="/example/MetalRoughSpheres">
            Metal Rough Spheres
          </SidebarButton>
          <SidebarButton href="/example/MetalRoughSpheresNoTextures">
            Metal Rough Spheres No Textures
          </SidebarButton>
          <SidebarButton href="/example/MorphPrimitivesTest">
            Morph Primitives Test
          </SidebarButton>
          <SidebarButton href="/example/MorphStressTest">
            Morph Stress Test
          </SidebarButton>
          <SidebarButton href="/example/MultiUVTest">
            Multi UV Test
          </SidebarButton>
          <SidebarButton href="/example/NormalTangentTest">
            Normal Tangent Test
          </SidebarButton>
          <SidebarButton href="/example/NormalTangentMirrorTest">
            Normal Tangent Mirror Test
          </SidebarButton>
          <SidebarButton href="/example/OrientationTest">
            Orientation Test
          </SidebarButton>
          <SidebarButton href="/example/RecursiveSkeletons">
            Recursive Skeletons
          </SidebarButton>
          <SidebarButton href="/example/TextureCoordinateTest">
            Texture Coordinate Test
          </SidebarButton>
          <SidebarButton href="/example/TextureLinearInterpolationTest">
            Texture Linear Interpolation Test
          </SidebarButton>
          <SidebarButton href="/example/TextureSettingsTest">
            Texture Settings Test
          </SidebarButton>
          <SidebarButton href="/example/VertexColorTest">
            Vertex Color Test
          </SidebarButton>
        </div>

        <div className="text-xl font-bold">Minimal Tests</div>

        <div className="space-y-2">
          <SidebarButton href="/example/TriangleWithoutIndices">
            Triangle Without Indices
          </SidebarButton>
          <SidebarButton href="/example/Triangle">Triangle</SidebarButton>
          <SidebarButton href="/example/AnimatedTriangle">
            Animated Triangle
          </SidebarButton>
          <SidebarButton href="/example/AnimatedMorphCube">
            Animated Morph Cube
          </SidebarButton>
          <SidebarButton href="/example/AnimatedMorphSphere">
            Animated Morph Sphere
          </SidebarButton>
          <SidebarButton href="/example/SimpleMeshes">
            Simple Meshes
          </SidebarButton>
          <SidebarButton href="/example/SimpleMorph">
            Simple Morph
          </SidebarButton>
          <SidebarButton href="/example/SimpleSparseAccessor">
            Simple Sparse Accessor
          </SidebarButton>
          <SidebarButton href="/example/SimpleSkin">Simple Skin</SidebarButton>
          <SidebarButton href="/example/InterpolationTest">
            Interpolation Test
          </SidebarButton>
          <SidebarButton href="/example/UnicodeTest">
            Unicode Test
          </SidebarButton>
        </div>
      </div>

      <div className="w-full">{children}</div>
    </div>
  );
}
