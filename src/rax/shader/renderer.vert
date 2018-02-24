#version 330
in vec2 position;
in vec4 color;

uniform float left = -1.0;
uniform float right = 1.0;
uniform float top = 1.0;
uniform float bottom = -1.0;

out FragData {
    vec2 position;
    vec4 color;
} frag;

void main() {
    vec2 lb = vec2(left, bottom);
    vec2 rt = vec2(right, top);
    vec2 translated = (position - lb) * 2.0 / (rt - lb) - vec2(1.0);
	gl_Position = vec4(translated, 0.0, 1.0);

	frag.position = position;
	frag.color = color;
}
