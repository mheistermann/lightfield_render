#version 330

in vec2 v_tex_coords;
out vec4 color;

uniform usampler2DArray tex;

void main() {
    color = texture(tex, vec3(v_tex_coords, 0));
    //color = vec4(v_tex_coords, 0, 0);
    //color = vec4(1,0.6,1,1);
}
