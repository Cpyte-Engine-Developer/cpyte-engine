use memoffset::offset_of;
use nalgebra::{Vector2, Vector3};
use std::mem::size_of;
use vulkanalia::vk::{
    Format, HasBuilder, VertexInputAttributeDescription, VertexInputBindingDescription,
    VertexInputRate,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Vertex {
    pos: Vector3<f32>,
    color: Vector3<f32>,
    texture_uv: Vector2<f32>,
}

impl Vertex {
    pub(crate) fn new(pos: Vector3<f32>, color: Vector3<f32>, texture_uv: Vector2<f32>) -> Self {
        Self {
            pos,
            color,
            texture_uv,
        }
    }

    pub(crate) fn binding_description() -> VertexInputBindingDescription {
        VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(VertexInputRate::VERTEX)
            .build()
    }

    pub(crate) fn attribute_descriptions() -> [VertexInputAttributeDescription; 3] {
        [
            VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, pos) as u32)
                .build(),
            VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, color) as u32)
                .build(),
            VertexInputAttributeDescription::builder()
                .binding(0)
                .location(2)
                .format(Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, texture_uv) as u32)
                .build(),
        ]
    }
}
