#version 450

// VERTEX
layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 texcoord;
// INSTANCE
layout (location=3) in uint texture_id;

layout (set=0, binding=0) uniform UniformBufferObject {
	mat4 view_matrix;
	mat4 projection_matrix;
} ubo;

layout( push_constant ) uniform PushConstants {
	mat4 model_matrix;
	mat4 inverse_model_matrix;
} pc;

layout (location=0) out uint tex_id;
layout (location=1) out vec2 uv;

void main() {
	vec4 worldpos = pc.model_matrix * vec4(position, 1.0);
    gl_Position = ubo.projection_matrix * ubo.view_matrix * worldpos;
    tex_id = texture_id;
    uv = texcoord;
}
