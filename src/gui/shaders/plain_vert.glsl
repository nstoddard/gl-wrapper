in vec2 pos;
in vec4 color;

out vec4 Color;

uniform mat4 matrix;
uniform vec4 uniColor;

void main() {
  Color = color * uniColor;
  writeGlPosition2D(matrix * vec4(pos, 0.0, 1.0));
}
