precision mediump float;

uniform vec2 u_resolution;

uniform vec3 border_color;
uniform float border_thickness;

varying vec2 v_coords;

void main() {
    vec2 coords = v_coords * u_resolution;

    float xl = step(coords.x, border_thickness);
    float yl = step(coords.y, border_thickness);

    float xr = step(u_resolution.x - border_thickness, coords.x);
    float yr = step(u_resolution.y - border_thickness, coords.y);

    float alpha = xl + yl + xr + yr;

    gl_FragColor = vec4(border_color * alpha, alpha);
}
