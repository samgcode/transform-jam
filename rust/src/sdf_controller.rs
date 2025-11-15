use godot::classes::file_access::ModeFlags;
// use crate::game_controller::GameController;
use godot::classes::rendering_device::UniformType;
use godot::classes::{
  FileAccess, IMeshInstance3D, Input, MeshInstance3D, RdShaderFile, RdUniform, RenderingDevice,
  RenderingServer, ShaderMaterial,
};
use godot::global::Key;
use godot::prelude::*;

const COLLISION_SHADER_PATH: &str = "res://collision.glsl";
const SHAPECAST_SHADER_PATH: &str = "res://shapecast.glsl";

const MAX_SHAPES: usize = 100;

const BLEND_FACTOR: &str = "BLEND_FACTOR";
const BACKGROUND: &str = "BACKGROUND_COLOR";
const POSITIONS: &str = "POSITIONS";
const PROPERTIES: &str = "PROPERTIES";
const COLORS: &str = "COLORS";

#[allow(unused)]
pub const FLAG_COLLISION: f32 = 0.0;
#[allow(unused)]
pub const FLAG_NO_COLLISION: f32 = 1.0;
#[allow(unused)]
pub const FLAG_NO_RENDER: f32 = 2.0;

#[derive(GodotClass)]
#[class(base = MeshInstance3D)]
pub struct SdfController {
  #[base]
  base: Base<MeshInstance3D>,

  #[export]
  blend_factor: f32,
  rendering_device: Gd<RenderingDevice>,

  background_color: ColorHsv,
  positions: PackedVector4Array,
  properties: PackedVector4Array,
  colors: PackedVector4Array,

  num_shapes: usize,
  shapes_used: [bool; MAX_SHAPES],
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
      positions: PackedArray::from([Vector4::ZERO; MAX_SHAPES]),
      properties: PackedArray::from([Vector4::ZERO; MAX_SHAPES]),
      colors: PackedArray::from([Vector4::ZERO; MAX_SHAPES]),
      num_shapes: 0,
      shapes_used: [false; MAX_SHAPES],
      rendering_device,
    };
  }

  fn ready(&mut self) {
    self.load_map();
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

    material.set_shader_parameter(BLEND_FACTOR, &self.blend_factor.to_variant());
    material.set_shader_parameter(BACKGROUND, &self.background_color.to_rgb().to_variant());
    material.set_shader_parameter(POSITIONS, &self.positions.to_variant());
    material.set_shader_parameter(PROPERTIES, &self.properties.to_variant());
    material.set_shader_parameter(COLORS, &self.colors.to_variant());

    if Input::singleton().is_key_pressed(Key::TAB) {
      self.print_map();
    }
    if Input::singleton().is_key_pressed(Key::Q) {
      self.load_map();
    }
  }
}

impl SdfController {
  fn load_map(&mut self) {
    self.positions = PackedArray::from([Vector4::ZERO; MAX_SHAPES]);
    self.properties = PackedArray::from([Vector4::ZERO; MAX_SHAPES]);
    self.colors = PackedArray::from([Vector4::ZERO; MAX_SHAPES]);
    self.num_shapes = 0;
    self.shapes_used = [false; MAX_SHAPES];

    let file = FileAccess::open("res://default_map.txt", ModeFlags::READ).unwrap();
    let content = file.get_as_text();
    let content = content.split("\n");

    for i in 0..content.len() {
      let line = content[i].clone();
      let properties = line.split("\t");
      if properties.len() == 4 {
        let mut split_properties = Vec::new();

        for j in 0..properties.len() {
          split_properties.push(properties[j].split(" "));
        }

        let shape_key = split_properties[0][0].clone();
        let position = split_properties[1].clone();
        let properties = split_properties[2].clone();
        let color = split_properties[3].clone();

        let mut shape = 0.0;
        if shape_key == GString::from("sphere") {
          shape = 1.0;
        } else if shape_key == GString::from("cube") {
          shape = 2.0;
        }

        let _ = self.new_shape(
          Vector4::new(
            position[1].to_string().parse::<f32>().unwrap(),
            position[2].to_string().parse::<f32>().unwrap(),
            position[3].to_string().parse::<f32>().unwrap(),
            0.0,
          ),
          Vector4::new(
            properties[1].to_string().parse::<f32>().unwrap(),
            properties[2].to_string().parse::<f32>().unwrap(),
            properties[3].to_string().parse::<f32>().unwrap(),
            shape,
          ),
          Vector4::new(
            color[1].to_string().parse::<f32>().unwrap(),
            color[2].to_string().parse::<f32>().unwrap(),
            color[3].to_string().parse::<f32>().unwrap(),
            0.0,
          ),
        );
      }
    }
  }

  fn print_map(&self) {
    godot_print!("current map file");
    for i in 0..MAX_SHAPES {
      let position = self.positions[i];
      let properties = self.properties[i];
      let color = self.colors[i];

      if properties.w == 0.0 || position.w >= 1.0 {
        continue;
      }

      let mut output = "".to_string();

      if properties.w == 1.0 {
        output = format!("{}sphere\t", output);
      } else if properties.w == 2.0 {
        output = format!("{}cube\t", output);
      }

      output = format!(
        "{}position {} {} {}\t",
        output, position.x, position.y, position.z
      );

      output = format!(
        "{}scale {} {} {}\t",
        output, properties.x, properties.y, properties.z
      );

      output = format!("{}color {} {} {}", output, color.x, color.y, color.z);

      godot_print!("{}", output);
    }
  }

  pub fn compute_collision(&mut self, points: PackedVector4Array) -> Vec<Vector4> {
    let shader_code = load::<RdShaderFile>(COLLISION_SHADER_PATH)
      .get_spirv()
      .unwrap();
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

    let mut events = Vec::new();
    for i in 0..(output.len() / 4) {
      events.push(Vector4::new(
        output[i * 4 + 0],
        output[i * 4 + 1],
        output[i * 4 + 2],
        output[i * 4 + 3],
      ));
    }
    return events;
  }

  pub fn compute_shapecast(
    &mut self,
    points: PackedVector4Array,
    velocity: Vector4,
  ) -> Vec<Vector4> {
    let shader_code = load::<RdShaderFile>(SHAPECAST_SHADER_PATH)
      .get_spirv()
      .unwrap();
    let collision_shader = self.rendering_device.shader_create_from_spirv(&shader_code);

    let mut points = points;
    points.push(velocity);

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
      .compute_list_dispatch(compute_list, points.len() as u32 - 1, 1, 1);
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

    let mut events = Vec::new();
    for i in 0..(output.len() / 4 - 1) {
      events.push(Vector4::new(
        output[i * 4 + 0],
        output[i * 4 + 1],
        output[i * 4 + 2],
        output[i * 4 + 3],
      ));
    }
    return events;
  }

  pub fn new_shape(
    &mut self,
    position: Vector4,
    properties: Vector4,
    color: Vector4,
  ) -> Result<usize, &'static str> {
    if self.num_shapes == MAX_SHAPES {
      return Err("Cannot allocate new shape, maximum amount of shapes allocated");
    }

    for i in 0..MAX_SHAPES {
      if !self.shapes_used[i] {
        self.positions[i] = position;
        self.properties[i] = properties;
        self.colors[i] = color;
        self.shapes_used[i] = true;
        self.num_shapes += 1;

        return Ok(i);
      }
    }

    return Err("Cannot allocate new shape, no shape slot available");
  }

  pub fn update_shape(
    &mut self,
    address: usize,
    position: Vector4,
    properties: Vector4,
    color: Vector4,
  ) {
    self.positions[address] = position;
    self.properties[address] = properties;
    self.colors[address] = color;
  }

  pub fn remove_shape(&mut self, address: usize) {
    if !self.shapes_used[address] {
      panic!("Shape double free");
    }

    self.positions[address] = Vector4::ZERO;
    self.properties[address] = Vector4::ZERO;
    self.colors[address] = Vector4::ZERO;
    self.shapes_used[address] = false;
    self.num_shapes -= 1;
  }

  // fn game_controller(&mut self) -> Gd<GameController> {
  //   return self
  //     .base_mut()
  //     .get_parent()
  //     .unwrap()
  //     .cast::<GameController>();
  // }
}
