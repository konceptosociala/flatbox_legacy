#version 450
#extension GL_EXT_debug_printf : enable

layout (location=0) in vec3 position;
//layout (location=1) in mat4 model_matrix;
//layout (location=5) in vec3 colour;

layout (set=0, binding=0) uniform UniformBufferObject {
	mat4 view_matrix;
	mat4 projection_matrix;
} ubo;

layout (push_constant) uniform PushConstants {
	mat4 model_matrix;
	vec3 colour;
} pcs;

layout (location=0) out vec4 colourdata_for_the_fragmentshader;

void main() {
    gl_Position = ubo.projection_matrix * ubo.view_matrix * pcs.model_matrix * vec4(position, 1.0);
    colourdata_for_the_fragmentshader = vec4(pcs.colour, 1.0);
}
