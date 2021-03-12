#version 300 es

precision mediump float;

in vec2 vUv;
out vec4 fragColor;

// uniform float time;
uniform sampler2D renderTexture;

void main() {
    vec4 texColor = texture(renderTexture, vUv);
    if(texColor.a < 0.1)
        discard;
    fragColor = texColor;
}