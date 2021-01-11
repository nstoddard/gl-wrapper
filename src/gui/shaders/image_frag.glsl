in vec2 UV;
in vec4 Color;

uniform sampler2D tex;

void main() {
  vec4 tex_color = Color * texture(tex, UV);
  // Premultiplied alpha
  tex_color.rgb *= tex_color.a;
  writeColor2D(tex_color);
}
