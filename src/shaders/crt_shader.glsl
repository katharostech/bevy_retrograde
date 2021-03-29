// Modified from:
// https://github.com/henriquelalves/SimpleGodotCRTShader/blob/d9a49f12fc1dce1c2ea04c157bc5ef4bff5e3b2e/CRTShader.shader#L1
// 
// Under the following license:
//
//     The MIT License (MIT)
//     
//     Copyright (c) 2016 Henrique Lacreta Alves
//     
//     Permission is hereby granted, free of charge, to any person obtaining a copy
//     of this software and associated documentation files (the "Software"), to deal
//     in the Software without restriction, including without limitation the rights
//     to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//     copies of the Software, and to permit persons to whom the Software is
//     furnished to do so, subject to the following conditions:
//     
//     The above copyright notice and this permission notice shall be included in all
//     copies or substantial portions of the Software.
//     
//     THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//     IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//     FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//     AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//     LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//     OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//     SOFTWARE.

in vec2 uv;

out vec4 frag_color;

uniform uvec2 window_size;
uniform float time;
uniform sampler2D screen_texture;

// Curvature
const float BarrelPower = {{CURVATURE}};
// Color bleeding
const float color_bleeding = {{COLOR_BLEEDING}};
const float bleeding_range_x = {{BLEEDING_RANGE_X}};
const float bleeding_range_y = {{BLEEDING_RANGE_Y}};
// Scanline
const float lines_distance = {{LINES_DISTANCE}};
const float scan_size = {{SCAN_SIZE}};
const float scanline_alpha = {{SCANLINE_ALPHA}};
const float lines_velocity = {{LINES_VELOCITY}};

vec2 distort(vec2 p) 
{
	float angle = p.y / p.x;
	float theta = atan(p.y,p.x);
	float radius = pow(length(p), BarrelPower);
	
	p.x = radius * cos(theta);
	p.y = radius * sin(theta);
	
	return 0.5 * (p + vec2(1.0,1.0));
}

void get_color_bleeding(inout vec4 current_color,inout vec4 color_left){
	current_color = current_color*vec4(color_bleeding,0.5,1.0-color_bleeding,1.0);
	color_left = color_left*vec4(1.0-color_bleeding,0.5,color_bleeding,1.0);
}

void get_color_scanline(vec2 uv,inout vec4 c,float time){
	float line_row = floor((uv.y * float(window_size.y)/scan_size) + mod(time*lines_velocity, lines_distance));
	float n = 1.0 - ceil((mod(line_row,lines_distance)/lines_distance));
	c = c - n*c*(1.0 - scanline_alpha);
	c.a = 1.0;
}

void main()
{
	vec2 xy = uv * 2.0;
	xy.x -= 1.0;
	xy.y -= 1.0;
	
	float d = length(xy);
	if(d < 1.5){
		xy = distort(xy);
	}
	else{
		xy = uv;
	}

	float pixel_size_x = 1.0/float(window_size.x)*bleeding_range_x;
	float pixel_size_y = 1.0/float(window_size.y)*bleeding_range_y;
	vec4 color_left = texture(screen_texture,xy - vec2(pixel_size_x, pixel_size_y));
	vec4 current_color = texture(screen_texture,xy);
	get_color_bleeding(current_color,color_left);
	vec4 c = current_color+color_left;
	get_color_scanline(xy,c,time);

    frag_color = c;
}