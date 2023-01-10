#version 450

// VERTEX
layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 texcoord;
// INSTANCE
layout (location=3) in vec3 colour;

layout (set=0, binding=0) uniform UniformBufferObject {
	mat4 view_matrix;
	mat4 projection_matrix;
} ubo;

layout( push_constant ) uniform PushConstants {
	mat4 model_matrix;
	mat4 inverse_model_matrix;
} pc;

layout (location=0) out vec3 out_colour;

void main() {
	vec4 worldpos = pc.model_matrix * vec4(position, 1.0);
    gl_Position = ubo.projection_matrix * ubo.view_matrix * worldpos;
}
