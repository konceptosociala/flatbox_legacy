#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout (location=0) out vec4 theColour;

layout (location=0) in vec2 uv;
layout (location=1) flat in uint texture_id;
layout (location=2) in vec3 normal;
layout (location=3) in vec3 worldpos;
layout (location=4) in vec3 camera_coordinates;
layout (location=5) in float metallic;
layout (location=6) in float roughness;

layout (set=1, binding=0) uniform sampler2D texturesamplers[];

readonly layout (set=2, binding=0) buffer StorageBufferObject {
	float num_directional;
	float num_point;
	vec3 data[];
} sbo;

struct DirectionalLight{
	vec3 direction_to_light;
	vec3 irradiance;
};

struct PointLight{
	vec3 position;
	vec3 luminous_flux;
};

const float PI = 3.1415926535897932;

void main(){
	vec3 lightColor = vec3(1.0);
	
	float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;
    	
	vec3 norm = normalize(normal);
	vec3 lightDir = normalize(sbo.data[0]);
	float diff = max(dot(norm, lightDir), 0.0);
	vec3 diffuse = diff * lightColor;
	vec3 result = (ambient + diffuse) * vec3(texture(texturesamplers[texture_id], uv));
	
	theColour = vec4(result, 1.0);
}
