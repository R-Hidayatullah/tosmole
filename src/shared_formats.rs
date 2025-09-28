//! Shared file format definitions for skeletal animation and motion data.
//!
//! This module provides type definitions and data structures for handling
//! various animation file formats including skeletal motions, actors, and
//! progressive morph motions.

use std::{
    fmt,
    io::{self, Read, Seek},
};

/// Type of skeletal motion data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SkeletalMotionType {
    /// A regular keyframe and keytrack based skeletal motion
    Normal = 0,
    /// A wavelet compressed skeletal motion
    Wavelet = 1,
}

/// File type identification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FileType {
    /// An unknown file, or something went wrong
    Unknown = 0,
    /// An actor file (.xac)
    Actor = 1,
    /// A skeletal motion file (.xsm)
    SkeletalMotion = 2,
    /// A wavelet compressed skeletal motion (.xsm)
    WaveletSkeletalMotion = 3,
    /// A progressive morph motion file (.xpm)
    PMorphMotion = 4,
}

/// Shared chunk identifiers
pub mod chunk_ids {
    pub const MOTION_EVENT_TABLE: u32 = 50;
    pub const TIMESTAMP: u32 = 51;
}

/// Matrix multiplication order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MultiplicationOrder {
    /// Scale, then rotate, then translate
    ScaleRotTrans = 0,
    /// Rotate, then scale, then translate
    RotScaleTrans = 1,
}

/// File chunk header
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileChunk {
    /// The chunk identifier
    pub chunk_id: u32,
    /// The size in bytes of this chunk (excluding this chunk struct)
    pub size_in_bytes: u32,
    /// The version of the chunk
    pub version: u32,
}

/// RGBA color with values in [0..1] range
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileColor {
    /// Red component
    pub r: f32,
    /// Green component
    pub g: f32,
    /// Blue component
    pub b: f32,
    /// Alpha component
    pub a: f32,
}

/// 3D vector with 32-bit floating point components
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct FileVector3 {
    /// X coordinate (positive = to the right)
    pub x: f32,
    /// Y coordinate (positive = up)
    pub y: f32,
    /// Z coordinate (positive = forwards into the depth)
    pub z: f32,
}

/// Compressed 3D vector with 16-bit integer components
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct File16BitVector3 {
    /// X coordinate (positive = to the right)
    pub x: u16,
    /// Y coordinate (positive = up)
    pub y: u16,
    /// Z coordinate (positive = forwards into the depth)
    pub z: u16,
}

/// Compressed 3D vector with 8-bit integer components
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct File8BitVector3 {
    /// X coordinate (positive = to the right)
    pub x: u8,
    /// Y coordinate (positive = up)
    pub y: u8,
    /// Z coordinate (positive = forwards into the depth)
    pub z: u8,
}

/// Quaternion with 32-bit floating point components
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileQuaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Compressed quaternion with 16-bit signed integer components
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct File16BitQuaternion {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub w: i16,
}

/// Motion event (version 1)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileMotionEvent {
    /// Time at which the event occurs
    pub time: f32,
    /// Index into the event type string table
    pub event_type_index: u32,
    /// Index into the parameter string table
    pub param_index: u32,
}

/// Motion event with start and end times (version 2)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileMotionEvent2 {
    /// Start time of the event
    pub start_time: f32,
    /// End time of the event
    pub end_time: f32,
    /// Index into the event type string table
    pub event_type_index: u32,
    /// Index into the parameter string table
    pub param_index: u32,
}

/// Timestamp information
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileTime {
    pub year: u16,
    pub month: i8,
    pub day: i8,
    pub hours: i8,
    pub minutes: i8,
    pub seconds: i8,
    padding: u8,
}

/// Motion event with optimized layout (version 3)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed(1))]
pub struct FileMotionEvent3 {
    /// Start time of the event
    pub start_time: f32,
    /// End time of the event
    pub end_time: f32,
    /// Index into the event type string table
    pub event_type_index: u32,
    /// Index into the parameter string table
    pub param_index: u16,
    padding: [u8; 2],
}

/// Motion event track for single-track file formats
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileMotionEventTrack {
    /// Number of events in this track
    pub num_events: u32,
    /// Number of type strings
    pub num_type_strings: u32,
    /// Number of parameter strings
    pub num_param_strings: u32,
    // Note: In the actual file format, this is followed by:
    // - [num_type_strings] string objects
    // - [num_param_strings] string objects
    // - FileMotionEvent3[num_events]
}

/// Motion event track with metadata (version 2)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileMotionEventTrack2 {
    /// Number of events in this track
    pub num_events: u32,
    /// Number of type strings
    pub num_type_strings: u32,
    /// Number of parameter strings
    pub num_param_strings: u32,
    /// Whether this track is enabled
    pub is_enabled: bool,
    padding: [u8; 3],
    // Note: In the actual file format, this is followed by:
    // - String track name
    // - [num_type_strings] string objects
    // - [num_param_strings] string objects
    // - FileMotionEvent3[num_events]
}

/// Motion event table containing multiple tracks
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileMotionEventTable {
    /// Number of tracks in this table
    pub num_tracks: u32,
    // Note: In the actual file format, this is followed by:
    // - FileMotionEventTrack2[num_tracks]
}

/// File attribute with dynamic data
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FileAttribute {
    /// Data type identifier
    pub data_type: u32,
    /// Size of the attribute data in bytes
    pub num_bytes: u32,
    /// Attribute flags
    pub flags: u32,
    // Note: In the actual file format, this is followed by:
    // - u8 data[num_bytes]
}

impl FileAttribute {
    pub fn new(data_type: u32, num_bytes: u32, flags: u32) -> Self {
        Self {
            data_type,
            num_bytes,
            flags,
        }
    }
}

// Type aliases for common use cases
pub type Vector3f = FileVector3;
pub type Vector3u16 = File16BitVector3;
pub type Vector3u8 = File8BitVector3;
pub type Quaternionf = FileQuaternion;
pub type Quaternioni16 = File16BitQuaternion;
pub type Color = FileColor;
