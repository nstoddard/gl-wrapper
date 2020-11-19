in vec2 pos;
in vec2 uv;
in vec4 color;

out vec2 UV;
out vec4 Color;

uniform mat4 matrix;
uniform vec4 uniColor;

void main() {
  UV = uv;
  Color = color * uniColor;
  writeGlPosition2D(matrix * vec4(pos, 0.0, 1.0));
}
