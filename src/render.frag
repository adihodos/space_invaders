#version 450 core

in VsOutFsIn {
  vec2 uv;
  vec4 color;
} fs_in;

layout (binding = 0) uniform sampler2D uTexture;

layout (location=0) out vec4 final_frag_color;

void main(void) {
  final_frag_color = fs_in.color * texture(uTexture, fs_in.uv);
}