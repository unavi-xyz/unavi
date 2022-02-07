import { useEffect, useState } from "react";
import { Grid, Stack, Typography } from "@mui/material";
import { Triplet } from "@react-three/cannon";
import { degToRad, radToDeg } from "three/src/math/MathUtils";
import { ASSETS, ASSET_NAMES, EditorObject } from "3d";

import { useStore } from "../../state/useStore";

import TripletField from "../components/fields/TripletField";
import ColorField from "../components/fields/ColorField";
import NumberField from "../components/fields/NumberField";
import { PROPERTIES } from "3d/packs/classes/Asset";
import SelectField from "../components/fields/SelectField";
import CheckField from "../components/fields/CheckField";

interface Props {
  object: EditorObject;
}

export default function Inspect({ object }: Props) {
  const { params } = object;
  const properties = ASSETS[params.type].properties;

  const usingGizmo = useStore((state) => state.usingGizmo);

  const [pos, setPos] = useState(params.position);
  const [rot, setRot] = useState(params.rotation);
  const [scale, setScale] = useState(params.scale);
  const [radius, setRadius] = useState(params.radius);
  const [color, setColor] = useState(params.color);
  const [opacity, setOpacity] = useState(params.opacity);
  const [physEnabled, setPhysEnabled] = useState(params.physEnabled);
  const [physType, setPhysType] = useState(params.physType);
  const [mass, setMass] = useState(params.mass);

  useEffect(() => {
    //object -> ui
    if (usingGizmo) return;

    object.save();

    const degrees = params.rotation.map(radToDeg) as Triplet;

    setPos(params.position);
    setRot(degrees);
    setScale(params.scale);

    setRadius(params.radius);

    setColor(params.color);
    setOpacity(params.opacity);

    setPhysEnabled(params.physEnabled);
    setPhysType(params.physType);
    setMass(params.mass);

    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [usingGizmo]);

  useEffect(() => {
    //ui -> object
    if (usingGizmo) return;

    const radians = rot.map(degToRad) as Triplet;

    params.position = pos;
    params.rotation = radians;

    if (params.type === ASSET_NAMES.Sphere) {
      params.scale = [radius * 2, radius * 2, radius * 2];
    } else {
      params.scale = scale;
    }

    params.radius = radius;

    params.color = color;
    params.opacity = opacity;

    params.physEnabled = physEnabled;
    params.physType = physType;
    params.mass = mass;

    object.load();
  }, [
    color,
    mass,
    object,
    opacity,
    params,
    physEnabled,
    physType,
    pos,
    properties,
    radius,
    rot,
    scale,
    usingGizmo,
  ]);

  return (
    <Stack spacing={3}>
      <Typography variant="h3">{params.name}</Typography>

      <div>
        {properties.includes(PROPERTIES.position) && (
          <TripletField title="Position" value={pos} setValue={setPos} />
        )}
        {properties.includes(PROPERTIES.rotation) && (
          <TripletField
            title="Rotation"
            step={1}
            value={rot}
            setValue={setRot}
          />
        )}
        {properties.includes(PROPERTIES.scale) && (
          <TripletField title="Scale" value={scale} setValue={setScale} />
        )}
      </div>

      {properties.includes(PROPERTIES.radius) && (
        <Grid container direction="row" alignItems="center">
          <Grid item xs={3}>
            <Typography>Radius</Typography>
          </Grid>
          <Grid item xs={9}>
            <NumberField min={0} value={radius} setValue={setRadius} />
          </Grid>
        </Grid>
      )}

      {properties.includes(PROPERTIES.color) ||
      properties.includes(PROPERTIES.opacity) ? (
        <>
          <Typography variant="h4">Material</Typography>

          <div>
            {properties.includes(PROPERTIES.color) && (
              <Grid container direction="row" alignItems="center">
                <Grid item xs={3}>
                  <Typography>Color</Typography>
                </Grid>
                <Grid item xs={9}>
                  <ColorField value={color} setValue={setColor} />
                </Grid>
              </Grid>
            )}

            {properties.includes(PROPERTIES.opacity) && (
              <Grid container direction="row" alignItems="center">
                <Grid item xs={3}>
                  <Typography>Opacity</Typography>
                </Grid>
                <Grid item xs={9}>
                  <NumberField
                    max={1}
                    min={0}
                    value={opacity}
                    setValue={setOpacity}
                  />
                </Grid>
              </Grid>
            )}
          </div>
        </>
      ) : null}

      {properties.includes(PROPERTIES.color) ||
      properties.includes(PROPERTIES.opacity) ? (
        <>
          <Typography variant="h4">Physics</Typography>

          <div>
            {properties.includes(PROPERTIES.physics) && (
              <Grid container direction="row" alignItems="center">
                <Grid item xs={3}>
                  <Typography>Enabled</Typography>
                </Grid>
                <Grid item xs={9}>
                  <CheckField value={physEnabled} setValue={setPhysEnabled} />
                </Grid>
              </Grid>
            )}

            {physEnabled && (
              <Grid container direction="row" alignItems="center">
                <Grid item xs={3}>
                  <Typography>Type</Typography>
                </Grid>
                <Grid item xs={9}>
                  <SelectField
                    options={["Static", "Dynamic"]}
                    value={physType}
                    setValue={setPhysType}
                  />
                </Grid>
              </Grid>
            )}

            {physEnabled && (
              <Grid container direction="row" alignItems="center">
                <Grid item xs={3}>
                  <Typography>Mass</Typography>
                </Grid>
                <Grid item xs={9}>
                  <NumberField min={0} value={mass} setValue={setMass} />
                </Grid>
              </Grid>
            )}
          </div>
        </>
      ) : null}
    </Stack>
  );
}
