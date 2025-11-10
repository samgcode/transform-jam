#[compute]
#version 450

const int NUM_SHAPES = 3;
const float EPSILON = 0.01;

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0, std430) restrict buffer PointBuffer {
  vec4 points[];
}
point_buffer;

layout(set = 0, binding = 1, std430) restrict buffer PositionBuffer {
  vec4 positions[];
}
position_buffer;

layout(set = 0, binding = 2, std430) restrict buffer PropertyBuffer {
  vec4 properties[];
}
property_buffer;

layout(set = 0, binding = 3, std430) restrict buffer DataBuffer {
  float blend_factor[];
}
data_buffer;


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
// <\SDF Operations>

float shape_dist(vec3 point, vec3 position, vec4 properties) {
	if(properties.w == 1.0) {
		return sdf_sphere(point - position, properties.x);
	}

	if(properties.w == 2.0) {
		return sdf_box(point - position, properties.xyz);
	}
}

float get_scene_dist(vec3 point) {
	float output_dist = 100.0;

	for(int i = 0; i < NUM_SHAPES; i++) {
		if(property_buffer.properties[i].w == 0.0 || position_buffer.positions[i].w == 1.0) continue;
		
		float dist = shape_dist(point, position_buffer.positions[i].xyz, property_buffer.properties[i]);

		output_dist = smoothUnion(output_dist, dist, data_buffer.blend_factor[0]);
	}
	return output_dist;
}

vec3 get_normal(vec3 point) {
	float dist = get_scene_dist(point);
	vec2 e = vec2(EPSILON, 0.0);
	vec3 normal = vec3(
		get_scene_dist(point + e.xyy) - get_scene_dist(point - e.xyy),
		get_scene_dist(point + e.yxy) - get_scene_dist(point - e.yxy),
		get_scene_dist(point + e.yyx) - get_scene_dist(point - e.yyx));
	return normal;
}

void main() {
  point_buffer.points[gl_GlobalInvocationID.x] = vec4(
    get_normal(point_buffer.points[gl_GlobalInvocationID.x].xyz),
    get_scene_dist(point_buffer.points[gl_GlobalInvocationID.x].xyz)
  );
}
