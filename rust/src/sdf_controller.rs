use crate::player::Player;
use godot::classes::rendering_device::UniformType;
use godot::classes::{
  IMeshInstance3D, MeshInstance3D, RdShaderFile, RdUniform, RenderingDevice, RenderingServer,
  ShaderMaterial,
};
use godot::prelude::*;

const SHADER_PATH: &str = "res://collision.glsl";

const BLEND_FACTOR: &str = "BLEND_FACTOR";
const BACKGROUND: &str = "BACKGROUND_COLOR";
const POSITIONS: &str = "POSITIONS";
const PROPERTIES: &str = "PROPERTIES";
const COLORS: &str = "COLORS";

#[allow(unused)]
const FLAG_COLLISION: f32 = 0.0;
#[allow(unused)]
const FLAG_NO_COLLISION: f32 = 1.0;
#[allow(unused)]
const FLAG_NO_RENDER: f32 = 2.0;

#[derive(GodotClass)]
#[class(base = MeshInstance3D)]
pub struct SdfController {
  #[base]
  base: Base<MeshInstance3D>,

  #[export]
  blend_factor: f32,
  rendering_device: Gd<RenderingDevice>,

  background_color: ColorHsv,
  #[export]
  positions: PackedVector4Array,
  #[export]
  properties: PackedVector4Array,
  #[export]
  colors: PackedVector4Array,
}

#[godot_api]
impl IMeshInstance3D for SdfController {
  fn init(base: Base<MeshInstance3D>) -> Self {
    let rendering_device = RenderingServer::singleton()
      .create_local_rendering_device()
      .unwrap();

    return Self {
      base,
      background_color: ColorHsv {
        h: 0.0,
        s: 1.0,
        v: 0.75,
        a: 1.0,
      },
      blend_factor: 1.0,
      positions: PackedArray::from(&[
        Vector4::new(0.0, 0.0, -1.5, FLAG_NO_RENDER),
        Vector4::new(0.0, -4.75, 0.0, FLAG_COLLISION),
        Vector4::new(0.0, -3.0, 0.0, FLAG_COLLISION),
        Vector4::new(3.0, -2.0, 0.0, FLAG_COLLISION),
        Vector4::new(0.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 0.0, 0.0),
      ]),
      properties: PackedArray::from(&[
        Vector4::new(0.02, 0.0, 0.0, 1.0),
        Vector4::new(3.0, 0.0, 0.0, 1.0),
        Vector4::new(5.0, 0.5, 5.0, 2.0),
        Vector4::new(1.0, 1.0, 5.0, 2.0),
        Vector4::new(0.01, 0.0, 0.0, 1.0),
        Vector4::new(0.01, 0.0, 0.0, 1.0),
        Vector4::new(0.01, 0.0, 0.0, 1.0),
        Vector4::new(0.01, 0.0, 0.0, 1.0),
        Vector4::new(0.01, 0.0, 0.0, 1.0),
        Vector4::new(0.01, 0.0, 0.0, 1.0),
      ]),
      colors: PackedArray::from(&[
        Vector4::new(1.0, 1.0, 1.0, 0.0),
        Vector4::new(1.0, 0.0, 0.0, 0.0),
        Vector4::new(0.0, 0.0, 1.0, 0.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, 1.0),
      ]),
      rendering_device,
    };
  }

  fn physics_process(&mut self, dt: f64) {
    if self.blend_factor < 0.0 {
      self.blend_factor = 0.0;
    }

    let mut material = self
      .base_mut()
      .get_mesh()
      .unwrap()
      .surface_get_material(0)
      .unwrap()
      .cast::<ShaderMaterial>();

    self.background_color.h += dt as f32 * 0.05;
    if self.background_color.h > 1.0 {
      self.background_color.h -= 1.0;
    }

    let player = self.base().get_node_as::<Player>("../../Player");

    let player_collider = player.bind().get_points();
    let collision_event = self.compute_collision(player_collider);

    if let Some(event) = collision_event {
      player.signals().collision().emit(event);
    }

    // {
    //   let player_collider = player.bind().get_points();
    //   self.positions[PLAYER_ID] = player_collider[0];
    //   self.positions[4] = player_collider[1];
    //   self.positions[5] = player_collider[2];
    //   self.positions[6] = player_collider[3];
    //   self.positions[7] = player_collider[4];
    //   self.positions[8] = player_collider[5];
    //   self.positions[9] = player_collider[6];
    // }

    material.set_shader_parameter(BLEND_FACTOR, &self.blend_factor.to_variant());
    material.set_shader_parameter(BACKGROUND, &self.background_color.to_rgb().to_variant());
    material.set_shader_parameter(POSITIONS, &self.positions.to_variant());
    material.set_shader_parameter(PROPERTIES, &self.properties.to_variant());
    material.set_shader_parameter(COLORS, &self.colors.to_variant());
  }
}

impl SdfController {
  fn compute_collision(&mut self, points: PackedVector4Array) -> Option<Vector4> {
    let shader_code = load::<RdShaderFile>(SHADER_PATH).get_spirv().unwrap();
    let collision_shader = self.rendering_device.shader_create_from_spirv(&shader_code);

    let point_bytes = points.to_byte_array();
    let position_bytes = self.positions.to_byte_array();
    let property_bytes = self.properties.to_byte_array();
    let data_bytes = PackedArray::from([self.blend_factor]).to_byte_array();

    let point_buffer = self
      .rendering_device
      .storage_buffer_create_ex(point_bytes.len() as u32)
      .data(&point_bytes)
      .done();

    let position_buffer = self
      .rendering_device
      .storage_buffer_create_ex(position_bytes.len() as u32)
      .data(&position_bytes)
      .done();

    let property_buffer = self
      .rendering_device
      .storage_buffer_create_ex(property_bytes.len() as u32)
      .data(&property_bytes)
      .done();

    let data_buffer = self
      .rendering_device
      .storage_buffer_create_ex(data_bytes.len() as u32)
      .data(&data_bytes)
      .done();

    let mut points_uniform = RdUniform::new_gd();
    points_uniform.set_uniform_type(UniformType::STORAGE_BUFFER);
    points_uniform.set_binding(0);
    points_uniform.add_id(point_buffer);

    let mut position_uniform = RdUniform::new_gd();
    position_uniform.set_uniform_type(UniformType::STORAGE_BUFFER);
    position_uniform.set_binding(1);
    position_uniform.add_id(position_buffer);

    let mut property_uniform = RdUniform::new_gd();
    property_uniform.set_uniform_type(UniformType::STORAGE_BUFFER);
    property_uniform.set_binding(2);
    property_uniform.add_id(property_buffer);

    let mut data_uniform = RdUniform::new_gd();
    data_uniform.set_uniform_type(UniformType::STORAGE_BUFFER);
    data_uniform.set_binding(3);
    data_uniform.add_id(data_buffer);

    let uniform_set = self.rendering_device.uniform_set_create(
      &Array::from(&[
        points_uniform,
        position_uniform,
        property_uniform,
        data_uniform,
      ]),
      collision_shader,
      0,
    );

    let pipeline = self
      .rendering_device
      .compute_pipeline_create(collision_shader);
    let compute_list = self.rendering_device.compute_list_begin();
    self
      .rendering_device
      .compute_list_bind_compute_pipeline(compute_list, pipeline);
    self
      .rendering_device
      .compute_list_bind_uniform_set(compute_list, uniform_set, 0);
    self
      .rendering_device
      .compute_list_dispatch(compute_list, points.len() as u32, 1, 1);
    self.rendering_device.compute_list_end();

    self.rendering_device.submit();
    self.rendering_device.sync();

    let output_bytes = self.rendering_device.buffer_get_data(point_buffer);
    let output = output_bytes.to_float32_array();

    self.rendering_device.free_rid(uniform_set);
    self.rendering_device.free_rid(point_buffer);
    self.rendering_device.free_rid(position_buffer);
    self.rendering_device.free_rid(property_buffer);
    self.rendering_device.free_rid(data_buffer);
    self.rendering_device.free_rid(pipeline);
    self.rendering_device.free_rid(collision_shader);

    let mut event = Vector4::ZERO;
    let mut highest_depth = 0.0;
    for i in 0..(output.len() / 4) {
      if output[i * 4 + 3] < 0.0 && output[i * 4 + 3] < highest_depth {
        highest_depth = output[i * 4 + 3];
        event = Vector4::new(
          output[i * 4 + 0],
          output[i * 4 + 1],
          output[i * 4 + 2],
          output[i * 4 + 3],
        );
      }
    }

    if highest_depth < 0.0 {
      return Some(event);
    }
    return None;
  }
}
