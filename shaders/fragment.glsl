#version 450

layout (location=0) out vec4 theColour;

layout (location=0) in vec3 colour_in;
layout (location=1) in vec3 normal;
layout (location=2) in vec3 worldpos;
layout (location=3) in vec3 camera_coordinates;
layout (location=4) in float metallic;
layout (location=5) in float roughness;

struct DirectionalLight{
	vec3 direction_to_light;
	vec3 irradiance;
};

struct PointLight{
	vec3 position;
	vec3 luminous_flux;
};

float PI = 3.1415926535897932;

float distribution(vec3 normal,vec3 halfvector,float roughness){
	float NdotH=dot(halfvector,normal);
	if (NdotH>0){
		float r=roughness*roughness;
		return r / (PI* (1 + NdotH*NdotH*(r-1))*(1 + NdotH*NdotH*(r-1)));
	}else{
		return 0.0;
	}
}

float geometry(vec3 light, vec3 normal, vec3 view, float roughness){
	float NdotL=abs(dot(normal,light));
	float NdotV=abs(dot(normal,view));
	return 0.5/max(0.01,mix(2*NdotL*NdotV,NdotL+NdotV,roughness));
}

vec3 compute_radiance(vec3 irradiance, vec3 light_direction, vec3 normal, vec3 camera_direction, vec3 surface_colour){
	float NdotL=	max(dot(normal,light_direction),0);
	
	vec3 irradiance_on_surface=irradiance*NdotL;

	float roughness2 = roughness*roughness;

	vec3 F0 = mix(vec3(0.03),surface_colour,vec3(metallic));
	vec3 reflected_irradiance = (F0 + (1 - F0)*(1-NdotL)*(1-NdotL)*(1-NdotL)*(1-NdotL)*(1-NdotL)) * irradiance_on_surface;
	vec3 refracted_irradiance = irradiance_on_surface - reflected_irradiance; 
	vec3 refracted_not_absorbed_irradiance = refracted_irradiance * (1-metallic);

	vec3 halfvector=normalize(0.5*(camera_direction + light_direction));
	float NdotH=max(dot(normal,halfvector),0);
	vec3 F=(F0 + (1 - F0)*(1-NdotH)*(1-NdotH)*(1-NdotH)*(1-NdotH)*(1-NdotH));
	vec3 relevant_reflection = reflected_irradiance*F*geometry(light_direction,normal,camera_direction,roughness2)*distribution(normal,halfvector,roughness2);

	return refracted_not_absorbed_irradiance*surface_colour/PI + relevant_reflection;
}

void main(){
	vec3 direction_to_camera = normalize(camera_coordinates - worldpos);
	vec3 normal = normalize(normal);
	vec3 L = vec3(0);

	DirectionalLight dlight = DirectionalLight(normalize(vec3(-1,-1,0)),vec3(0.5, 0.5, 0.5));

	L += compute_radiance(dlight.irradiance, dlight.direction_to_light, normal, direction_to_camera, colour_in);

	const int NUMBER_OF_POINTLIGHTS = 2;
	
	PointLight pointlights [NUMBER_OF_POINTLIGHTS] = { 
		PointLight(vec3(0.1,-3.0,-3.0),vec3(25,25,25)),
		PointLight(vec3(1.5, 0.0, 0.0),vec3(5,5,5)),
	};
	
	const float PI = 3.14159265358979323846264;	
	for (int i=0; i<NUMBER_OF_POINTLIGHTS; i++){
		PointLight light = pointlights[i];
		vec3 direction_to_light = normalize(light.position - worldpos);
		float d = length(worldpos - light.position);
		vec3 irradiance = light.luminous_flux/(4*PI*d*d);

		L += compute_radiance(irradiance, direction_to_light, normal, direction_to_camera, colour_in);
	};

	theColour=vec4(L/(1+L),1.0);
}

