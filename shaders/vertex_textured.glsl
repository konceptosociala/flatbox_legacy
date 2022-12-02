#version 450

layout (location=0) in vec3 position;
layout (location=1) in vec2 texcoord;
layout (location=2) in mat4 model_matrix;
layout (location=6) in mat4 inverse_model_matrix;
layout (location=10) in uint texture_id;

layout (set=0, binding=0) uniform UniformBufferObject {
	mat4 view_matrix;
	mat4 projection_matrix;
} ubo;

layout (location=0) out vec2 uv;
layout (location=1) out uint tex_id;

void main() {
    vec4 worldpos = model_matrix * vec4(position, 1.0);
    gl_Position = ubo.projection_matrix * ubo.view_matrix * worldpos;
    uv = texcoord;
    tex_id = texture_id;
}
