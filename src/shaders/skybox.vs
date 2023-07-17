#version 450

layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 texcoord;

layout (set=0, binding=0) uniform UniformBufferObject {
    mat4 view_matrix;
    mat4 projection_matrix;
} ubo;

layout( push_constant ) uniform PushConstants {
    mat4 model_matrix;
    mat4 inverse_model_matrix;
} pc;

layout (location=0) out vec3 outUVW;

void main() {
	outUVW = position;
	outUVW.xy *= -1.0;

    vec4 worldpos = pc.model_matrix * vec4(position.xyz, 1.0);
    gl_Position = ubo.projection_matrix * ubo.view_matrix * worldpos;
}