//! Shared file format definitions for skeletal animation and motion data.
//!
//! This module provides type definitions and data structures for handling
//! various animation file formats including skeletal motions, actors, and
//! progressive morph motions.

use std::fmt;

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

impl Default for FileColor {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
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

impl FileVector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
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

impl File16BitVector3 {
    pub fn new(x: u16, y: u16, z: u16) -> Self {
        Self { x, y, z }
    }
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

impl File8BitVector3 {
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        Self { x, y, z }
    }
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

impl Default for FileQuaternion {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0, // Identity quaternion
        }
    }
}

impl FileQuaternion {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn identity() -> Self {
        Self::default()
    }
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

impl File16BitQuaternion {
    pub fn new(x: i16, y: i16, z: i16, w: i16) -> Self {
        Self { x, y, z, w }
    }
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

impl FileTime {
    pub fn new(year: u16, month: i8, day: i8, hours: i8, minutes: i8, seconds: i8) -> Self {
        Self {
            year,
            month,
            day,
            hours,
            minutes,
            seconds,
            padding: 0,
        }
    }
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

impl FileMotionEvent3 {
    pub fn new(start_time: f32, end_time: f32, event_type_index: u32, param_index: u16) -> Self {
        Self {
            start_time,
            end_time,
            event_type_index,
            param_index,
            padding: [0; 2],
        }
    }
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

impl FileMotionEventTrack2 {
    pub fn new(
        num_events: u32,
        num_type_strings: u32,
        num_param_strings: u32,
        is_enabled: bool,
    ) -> Self {
        Self {
            num_events,
            num_type_strings,
            num_param_strings,
            is_enabled,
            padding: [0; 3],
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quaternion_identity() {
        let q = FileQuaternion::identity();
        assert_eq!(q.w, 1.0);
        assert_eq!(q.x, 0.0);
        assert_eq!(q.y, 0.0);
        assert_eq!(q.z, 0.0);
    }

    #[test]
    fn test_vector3_creation() {
        let v = FileVector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_file_time_creation() {
        let time = FileTime::new(2025, 8, 11, 14, 30, 45);
        assert_eq!(time.year, 2025);
        assert_eq!(time.month, 8);
        assert_eq!(time.day, 11);
    }
}
