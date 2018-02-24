#version 330

out vec4 out_color;

in FragData {
    vec2 position;
    vec4 color;
} frag;

void main() {
	out_color = frag.color;
}
