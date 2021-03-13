# version  300 es

layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aUv;

out vec2 vUv;

uniform float aspectCorrectionFactor;

void main() {
    vUv = aUv; 
    gl_Position = vec4(aPos.x / aspectCorrectionFactor, aPos.y, aPos.z, 1.0);
}