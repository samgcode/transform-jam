shader_type spatial;
render_mode unshaded, depth_test_disabled;

const int MAX_SHAPES = 10;

uniform int MAX_STEPS = 100;
uniform float MAX_DIST = 100.0;
uniform float SURF_DIST = 0.01;
uniform float BLEND_FACTOR: hint_range(0, 2.0) = 1.0;

uniform vec3 BACKGROUND_COLOR : source_color = vec3(0.0);

uniform vec4 POSITIONS[MAX_SHAPES]; // vec4(pos.xyz, flag) flag 0.0 means collidable, 1.0 no collision, 2.0 means no render
uniform vec4 PROPERTIES[MAX_SHAPES]; // vec4(dimensions.xyz, type) type 0.0 means no object
uniform vec3 COLORS[MAX_SHAPES];

// <SDF Primitives>
float sdf_sphere(vec3 point, float r) {
	return length(point) - r;
}

float sdf_box(vec3 point, vec3 bounds) {
  vec3 q = abs(point) - bounds;
  return length(max(q, 0.0)) + min(max(q.x, max(q.y,q.z)), 0.0);
}
// <\SDF Primitives>

// <SDF Operations>
float smoothUnion(float dist1, float dist2, float k) {
	float h = clamp(0.5 + 0.5 * (dist2 - dist1) / k, 0.0, 1.0);
	return mix(dist2, dist1, h) - k * h * (1.0 - h);
}

vec4 smoothUnion(vec4 a, vec4 b, float k) {
	float h = clamp(0.5 + 0.5 * (b.w - a.w) / k, 0.0, 1.0);
	return vec4(
		mix(b.rgb, a.rgb, h),
		mix(b.w, a.w, h) - k * h * (1.0 - h)
	);
}
// <\SDF Operations>

float shape_dist(vec3 point, vec3 position, vec4 properties) {
	if(properties.w == 1.0) {
		return sdf_sphere(point - position, properties.x);
	}

	if(properties.w == 2.0) {
		return sdf_box(point - position, properties.xyz);
	}
}

// returns vec4(r, g, b, dist)
vec4 get_scene_info(vec3 point) {
	vec4 output_info = vec4(1.0, 1.0, 1.0, MAX_DIST);

	for(int i = 0; i < MAX_SHAPES; i++) {
		if(PROPERTIES[i].w == 0.0 || POSITIONS[i].w == 2.0) continue;
		
		vec3 color = COLORS[i];
		float dist = shape_dist(point, POSITIONS[i].xyz, PROPERTIES[i]);

		output_info = smoothUnion(output_info, vec4(color.rgb, dist), BLEND_FACTOR);
	}
	return output_info;
}

float get_dist(vec3 point) {
	return get_scene_info(point).w;
}

vec3 get_normal(vec3 point) {
	float dist = get_dist(point);
	vec2 e = vec2(0.0001, 0.0);
	vec3 normal = dist - vec3(
		get_dist(point - e.xyy),
		get_dist(point - e.yxy),
		get_dist(point - e.yyx));
	return normalize(normal);
}

void fragment() {
	// generate the ray for the current pixel
	vec3 ws_pixel_pos = ((INV_VIEW_MATRIX * vec4(VERTEX, 1.0)).xyz);
	vec3 ray_origin = CAMERA_POSITION_WORLD;
	vec3 ray_dir = normalize(ws_pixel_pos - ray_origin);

	float total_dist = 0.0;
	vec3 color = BACKGROUND_COLOR;
	for(int i = 0; i < MAX_STEPS; i++) {
		vec3 point = ray_origin + ray_dir * total_dist;
		vec4 scene_info = get_scene_info(point);
		total_dist += scene_info.w;
		color = scene_info.rgb;

		if(total_dist > MAX_DIST || scene_info.w < SURF_DIST) break;
	}

	if(total_dist >= MAX_DIST) {
		ALBEDO = BACKGROUND_COLOR;
	} else {
		vec3 hit_point = ray_origin + ray_dir * total_dist;
		vec3 normal = get_normal(hit_point);

		float shade = dot(-ray_dir, normal);

		ALBEDO = color * shade;
	}
}
