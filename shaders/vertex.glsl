#version 450
#extension GL_EXT_debug_printf : enable

layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;

layout (set=0, binding=0) uniform UniformBufferObject {
	mat4 view_matrix;
	mat4 projection_matrix;
} ubo;

layout (push_constant) uniform PushConstants {
	mat4 model_matrix;
	mat4 inverse_model_matrix;
} pcs;

layout (location=0) out vec3 colourdata_for_the_fragmentshader;
layout (location=1) out vec3 out_normal;
layout (location=2) out vec4 worldpos;
layout (location=3) out vec3 camera_coordinates;

void main() {
	worldpos = pcs.model_matrix * vec4(position, 1.0);
    gl_Position = ubo.projection_matrix * ubo.view_matrix * worldpos;
    colourdata_for_the_fragmentshader = vec3(1.0, 1.0, 1.0);
    out_normal = transpose(mat3(pcs.inverse_model_matrix)) * normal;
    camera_coordinates =
		- ubo.view_matrix[3][0] * vec3 (ubo.view_matrix[0][0],ubo.view_matrix[1][0],ubo.view_matrix[2][0])
		- ubo.view_matrix[3][1] * vec3 (ubo.view_matrix[0][1],ubo.view_matrix[1][1],ubo.view_matrix[2][1])
		- ubo.view_matrix[3][2] * vec3 (ubo.view_matrix[0][2],ubo.view_matrix[1][2],ubo.view_matrix[2][2]);
}
