use std::fmt::Debug;
use nalgebra::Vector3;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ColorParseError {

}

#[derive(Default, Debug, Clone, Copy, PartialEq, Hash)]
pub struct Color<V> {
    pub r: V,
    pub g: V,
    pub b: V,
}

impl<V> Color<V> {
    pub const fn new(r: V, g: V, b: V) -> Self {
        Color { r, g, b }
    }
}

impl Color<u8> {
    pub const WHITE: Color<u8> = Color::new(255, 255, 255);
    pub const BLACK: Color<u8> = Color::new(0, 0, 0);
    pub const RED: Color<u8> = Color::new(200, 0, 0);
    pub const GREEN: Color<u8> = Color::new(0, 200, 0);
    pub const BLUE: Color<u8> = Color::new(0, 0, 200);
    pub const NORMAL: Color<u8> = Color::new(128, 128, 255);
}

impl<V: Clone> Color<V> {
    pub fn grayscale(value: V) -> Self {
        Color {
            r: value.clone(),
            g: value.clone(),
            b: value,
        }
    }
}

impl From<Color<f32>> for Color<u8> {
    fn from(value: Color<f32>) -> Self {
        Color {
            r: (value.r * 255.0) as u8,
            g: (value.g * 255.0) as u8,
            b: (value.b * 255.0) as u8,
        }
    }
}

impl From<Color<u8>> for Color<f32> {
    fn from(value: Color<u8>) -> Self {
        Color {
            r: (value.r as f32) / 255.0,
            g: (value.g as f32) / 255.0,
            b: (value.b as f32) / 255.0,
        }
    }
}

impl From<Vector3<f32>> for Color<f32> {
    fn from(value: Vector3<f32>) -> Self {
        Color {
            r: value.x,
            g: value.y,
            b: value.z,
        }
    }
}

impl From<Vector3<u8>> for Color<u8> {
    fn from(value: Vector3<u8>) -> Self {
        Color {
            r: value.x,
            g: value.y,
            b: value.z,
        }
    }
}

impl From<Color<u8>> for [u8; 4] {
    fn from(color: Color<u8>) -> Self {
        [color.r, color.g, color.b, 255]
    }
}

impl<V: Serialize> Serialize for Color<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer 
    {
        (&self.r, &self.g, &self.b).serialize(serializer)
    }
}

impl<'de, V: Deserialize<'de>> Deserialize<'de> for Color<V> {
    fn deserialize<D>(deserializer: D) -> Result<Color<V>, D::Error>
        where D: serde::Deserializer<'de>
    {
        Deserialize::deserialize(deserializer)
            .map(|(r, g, b)| Color { r, g, b })
    }
}