use crate::player::Player;
use godot::classes::rendering_device::UniformType;
use godot::classes::{
  IMeshInstance3D, MeshInstance3D, RdShaderFile, RdUniform, RenderingDevice, RenderingServer,
  ShaderMaterial,
};
use godot::prelude::*;

const SHADER_PATH: &str = "res://collision.glsl";

const BACKGROUND: &str = "BACKGROUND_COLOR";
const POSITIONS: &str = "POSITIONS";
const PROPERTIES: &str = "PROPERTIES";
const COLORS: &str = "COLORS";

const PLAYER_ID: usize = 0;
const GRAVITY: Vector3 = Vector3 {
  x: 0.0,
  y: -0.05,
  z: 0.0,
};

#[derive(GodotClass)]
#[class(base = MeshInstance3D)]
pub struct SdfController {
  #[base]
  base: Base<MeshInstance3D>,

  background_color: ColorHsv,
  positions: PackedVector4Array,
  properties: PackedVector4Array,
  colors: PackedColorArray,

  player: Player,

  rendering_device: Gd<RenderingDevice>,
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
      positions: PackedArray::from(&[
        Vector4::new(0.0, 0.0, -1.5, 0.0),
        Vector4::new(0.0, -4.75, 0.0, 0.0),
        Vector4::new(0.0, -3.0, 0.0, 0.0),
      ]),
      properties: PackedArray::from(&[
        Vector4::new(0.1, 0.0, 0.0, 1.0),
        Vector4::new(3.0, 0.0, 0.0, 1.0),
        Vector4::new(5.0, 0.5, 5.0, 2.0),
      ]),
      colors: PackedArray::from(&[
        Color::from_rgb(1.0, 1.0, 1.0),
        Color::from_rgb(1.0, 0.0, 0.0),
        Color::from_rgb(0.0, 0.0, 1.0),
      ]),
      player: Player::init(),
      rendering_device,
    };
  }

  fn physics_process(&mut self, dt: f64) {
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

    self.player.update(dt as f32, GRAVITY);

    let player_pos = Vector4::new(
      self.player.position.x,
      self.player.position.y,
      self.player.position.z,
      0.0,
    );

    let collision = self.compute_collision(
      PackedArray::from(&[player_pos]),
      PackedArray::from(&[self.positions[1], self.positions[2]]),
      PackedArray::from(&[self.properties[1], self.properties[2]]),
    );

    if collision.w < 0.0 {
      self.player.on_collision(collision, dt as f32);
    }

    let player_pos = Vector4::new(
      self.player.position.x,
      self.player.position.y,
      self.player.position.z,
      0.0,
    );

    self.positions[PLAYER_ID] = player_pos;

    material.set_shader_parameter(BACKGROUND, &self.background_color.to_rgb().to_variant());
    material.set_shader_parameter(POSITIONS, &self.positions.to_variant());
    material.set_shader_parameter(PROPERTIES, &self.properties.to_variant());
    material.set_shader_parameter(COLORS, &self.colors.to_variant());
  }
}

impl SdfController {
  fn compute_collision(
    &mut self,
    points: PackedVector4Array,
    positions: PackedVector4Array,
    properties: PackedVector4Array,
  ) -> Vector4 {
    let shader_code = load::<RdShaderFile>(SHADER_PATH).get_spirv().unwrap();
    let collision_shader = self.rendering_device.shader_create_from_spirv(&shader_code);

    let point_bytes = points.to_byte_array();
    let position_bytes = positions.to_byte_array();
    let property_bytes = properties.to_byte_array();

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

    let uniform_set = self.rendering_device.uniform_set_create(
      &Array::from(&[points_uniform, position_uniform, property_uniform]),
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
    self.rendering_device.free_rid(pipeline);
    self.rendering_device.free_rid(collision_shader);

    return Vector4::new(output[0], output[1], output[2], output[3]);
  }
}
