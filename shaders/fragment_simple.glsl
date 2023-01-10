#version 450

layout (location=0) out vec4 theColour;

layout (location=0) in vec3 colour;

void main() {
	theColour = vec4(colour, 1.0);
}
