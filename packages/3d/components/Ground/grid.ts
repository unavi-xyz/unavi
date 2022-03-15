export const grid_vertex = `
precision highp float;
precision highp int;

varying vec3 vPosition;
varying vec4 worldPosition;

void main() {
  vPosition = position;
  worldPosition = modelMatrix * vec4(position, 1.0);

  gl_Position = projectionMatrix * modelViewMatrix * vec4( position, 1.0 );
}
`;

export const grid_fragment = `
precision highp float;
precision highp int;

varying vec3 vPosition;
varying vec4 worldPosition;

// Some useful functions
vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
vec2 mod289(vec2 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
vec3 permute(vec3 x) { return mod289(((x*34.0)+1.0)*x); }

//
// Description : GLSL 2D simplex noise function
//      Author : Ian McEwan, Ashima Arts
//  Maintainer : ijm
//     Lastmod : 20110822 (ijm)
//     License :
//  Copyright (C) 2011 Ashima Arts. All rights reserved.
//  Distributed under the MIT License. See LICENSE file.
//  https://github.com/ashima/webgl-noise
//
float snoise(vec2 v) {
  // Precompute values for skewed triangular grid
  const vec4 C = vec4(0.211324865405187,
                      // (3.0-sqrt(3.0))/6.0
                      0.366025403784439,
                      // 0.5*(sqrt(3.0)-1.0)
                      -0.577350269189626,
                      // -1.0 + 2.0 * C.x
                      0.024390243902439);
                      // 1.0 / 41.0

  // First corner (x0)
  vec2 i  = floor(v + dot(v, C.yy));
  vec2 x0 = v - i + dot(i, C.xx);

  // Other two corners (x1, x2)
  vec2 i1 = vec2(0.0);
  i1 = (x0.x > x0.y)? vec2(1.0, 0.0):vec2(0.0, 1.0);
  vec2 x1 = x0.xy + C.xx - i1;
  vec2 x2 = x0.xy + C.zz;

  // Do some permutations to avoid
  // truncation effects in permutation
  i = mod289(i);
  vec3 p = permute(
          permute( i.y + vec3(0.0, i1.y, 1.0))
              + i.x + vec3(0.0, i1.x, 1.0 ));

  vec3 m = max(0.5 - vec3(
                      dot(x0,x0),
                      dot(x1,x1),
                      dot(x2,x2)
                      ), 0.0);

  m = m*m ;
  m = m*m ;

  // Gradients:
  //  41 pts uniformly over a line, mapped onto a diamond
  //  The ring size 17*17 = 289 is close to a multiple
  //      of 41 (41*7 = 287)

  vec3 x = 2.0 * fract(p * C.www) - 1.0;
  vec3 h = abs(x) - 0.5;
  vec3 ox = floor(x + 0.5);
  vec3 a0 = x - ox;

  // Normalise gradients implicitly by scaling m
  // Approximation of: m *= inversesqrt(a0*a0 + h*h);
  m *= 1.79284291400159 - 0.85373472095314 * (a0*a0+h*h);

  // Compute final noise value at P
  vec3 g = vec3(0.0);
  g.x  = a0.x  * x0.x  + h.x  * x0.y;
  g.yz = a0.yz * vec2(x1.x,x2.x) + h.yz * vec2(x1.y,x2.y);
  return 130.0 * dot(m, g);
}

void main() {
  vec2 coord = vPosition.xz;
  vec2 st = worldPosition.xz;

  vec2 grid = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
  float line = max(min(grid.x, grid.y), 0.8);

  float v1 = 317.0 * 3.0;
  float v2 = 487.0 * 3.0;
  float v3 = 613.0 * 3.0;
  float v4 = 751.0;

  float c1 = snoise(st / v1) * 0.5 + 0.5;
  float c2 = snoise(st / v2) * 0.5 + 0.5;
  float c3 = snoise(st / v3) * 0.5 + 0.5;
  float c4 = snoise(st / v4) * 0.5 + 0.5;

  vec3 baseColor = vec3(0.15);
  vec3 lineColor = mix(vec3(c1, c2, c3), baseColor, 0.7);

  if ( line > 1.0 ) {
    gl_FragColor = vec4(baseColor, 1.0);
  } else { 
    gl_FragColor = vec4(lineColor, 1.0);
  }
}
`;

export const grid_fragment_light = `
precision highp float;
precision highp int;

varying vec3 vPosition;

void main() {
  vec2 coord = vPosition.xy;

  vec2 grid = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
  float line = max(min(grid.x, grid.y), 0.97);

  gl_FragColor = vec4(vec3(min(line, 1.0)), 1.0);
}
`;

export const grid_fragment_dark = `
precision highp float;
precision highp int;

varying vec3 vPosition;

void main() {
  vec2 coord = vPosition.xy;

  vec2 grid = abs(fract(coord - 0.5) - 0.5) / fwidth(coord);
  float line = max(min(grid.x, grid.y), 0.8);

  gl_FragColor = vec4(vec3(1.0 - min(line, 0.85)), 1.0);
}
`;
