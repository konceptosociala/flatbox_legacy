#version 450

layout (location=0) out vec4 theColour;

layout (location=0) in vec3 inUVW;

// layout (set=3, binding=0) uniform samplerCube samplerCubeMap;

void main() 
{
	theColour = vec4(1.0, 0.0, 0.0, 1.0);
	// theColour = texture(samplerCubeMap, inUVW);
}