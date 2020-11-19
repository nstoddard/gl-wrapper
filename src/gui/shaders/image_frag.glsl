in vec2 UV;
in vec4 Color;

uniform sampler2D tex;

void main() {
  writeColor2D(Color * texture(tex, UV));
}
