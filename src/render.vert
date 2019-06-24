#version 450 core

layout (location=0) in vec2 inVertexPos;
layout (location=1) in vec2 inVertexUV;
layout (location=2) in vec4 inVertexColor;

layout (location = 0) uniform mat4 uMat4WorldViewProj;

out VsOutFsIn {
  vec2 uv;
  vec4 color;
} vs_out_fs_in;

void main(void) {
  gl_Position = uMat4WorldViewProj * vec4(inVertexPos, 0.0, 1.0);
  vs_out_fs_in.uv = inVertexUV;
  vs_out_fs_in.color = inVertexColor;
}
