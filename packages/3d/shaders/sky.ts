export const sky_vertex = `
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

export const sky_fragment = `
precision highp float;
precision highp int;

varying vec3 vPosition;
varying vec4 worldPosition;

void main() {
  float offset = 0.0;
  float exponent = 1.0;

  vec4 topColor = vec4(.7, .8, .9, 1.0);
  vec4 bottomColor = vec4(.2, .2, .4, 1.0);

  float h = normalize( worldPosition + offset ).y;


  vec4 color = vec4(mix( bottomColor, topColor, max(pow(max(h, 0.0), exponent), 0.0) ));

  gl_FragColor = color;
}
`;
