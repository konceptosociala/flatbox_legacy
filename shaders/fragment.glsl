#version 450
	
layout (location=0) out vec4 theColour;
layout (location=0) in vec4 colordata_from_the_vertexshader;

void main(){
	theColour = colordata_from_the_vertexshader;
}
