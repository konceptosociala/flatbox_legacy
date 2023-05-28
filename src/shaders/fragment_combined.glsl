#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout (location=0) out vec4 theColour;

layout (location=0) in vec2 uv;
layout (location=1) in vec3 normal;
layout (location=2) in vec3 worldpos;
layout (location=3) in vec3 camera_coordinates;

layout (location=4) in vec3 color;
layout (location=5) in flat uint albedo_map;
layout (location=6) in float metallic;
layout (location=7) in flat uint metallic_map;
layout (location=8) in float roughness;
layout (location=9) in flat uint roughness_map;
layout (location=10) in float normal_t;
layout (location=11) in flat uint normal_t_map;

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

void main(){ // TODO: Physically based rendering

    int number_directional = int(sbo.num_directional);
	int number_point = int(sbo.num_point);

    for(int i = 0; i < number_directional; i++) {
        vec3 direction = sbo.data[2 * i];
		vec3 illuminance = sbo.data[2 * i + 1];
		DirectionalLight dlight = DirectionalLight(normalize(direction), illuminance);
    }

    for(int i = 0; i < number_point; i++) {
        vec3 position = sbo.data[2 * i + 2 * number_point];
		vec3 luminous_flux = sbo.data[2 * i + 1 + 2 * number_point];
		PointLight plight = PointLight(position, luminous_flux);
    }

	vec3 lightColor = vec3(1.0);	
    
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;
  	
    // diffuse 
    vec3 norm = normalize(normal);
    vec3 lightDir = normalize(sbo.data[0]);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;
    
    // specular
    float specularStrength = 0.5;
    vec3 viewDir = normalize(camera_coordinates - worldpos);
    vec3 reflectDir = reflect(-lightDir, norm);  
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), 256);
    vec3 specular = specularStrength * spec * lightColor;  
        
    vec3 result = (ambient + diffuse + specular) * vec3(texture(texturesamplers[albedo_map], uv)) * color;
    theColour = vec4(result, 1.0);
}
