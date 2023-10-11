# XSM File Structure

The **XSM file format** is used to store skeletal motion data, such as bone animations, in various applications,
including game development. This document explains the binary structure of XSM files, with an emphasis on the header and
its key components.

## XSM Header

An XSM file begins with a header that contains essential information about the structure and content of the file.

### Header Fields

- **Magic Number (4 bytes):** A unique identifier or magic number that signifies the file type. It should be `XSM ` for
  a valid XSM file.

- **Major Version (1 byte):** The major version of the XSM file format.

- **Minor Version (1 byte):** The minor version of the XSM file format.

- **Big Endian (1 byte):** A boolean flag indicating whether the file is in big-endian format (usually false for
  little-endian systems).

- **Unused (1 byte):** Reserved for future use.

### Header Validation

- The magic number is a unique identifier for the XSM file format.

- The major and minor versions provide information about the file format's compatibility.

## Chunk Structure

Following the header, an XSM file contains a series of chunks, each with a specific type, length, and version. Chunks
represent various components of skeletal motion data.

### Chunk Components

Each chunk consists of the following components:

- **Chunk Type (4 bytes):** An identifier that indicates the type of the chunk. It corresponds to specific components,
  such as metadata and bone animations.

- **Length (4 bytes):** The length of the chunk, specifying the size of the data contained within the chunk.

- **Version (4 bytes):** The version of the chunk format.

### Chunk Processing

The XSM file parser processes each chunk based on its type, reading and interpreting the data within the chunk.

## Metadata Chunk

One of the common chunk types found in XSM files is the metadata chunk, which contains information about the skeletal
motion.

### Metadata Components

- **Unused (4 bytes):** Reserved for future use.

- **Max Acceptable Error (4 bytes):** A floating-point value that represents the maximum acceptable error.

- **Frames Per Second (4 bytes):** An integer value indicating the frames per second of the animation.

- **Exporter Version (2 bytes):** The major and minor versions of the exporter used to create the XSM file.

- **Padding (2 bytes):** Reserved for alignment.

- **Source Application (string):** The name of the application that generated the XSM file.

- **Original Filename (string):** The original filename before export.

- **Export Date (string):** The date of the export.

- **Motion Name (string):** The name of the skeletal motion.

## Bone Animation Chunk

Another common chunk in XSM files is the bone animation chunk, which contains data related to bone animations.

### Bone Animation Components

- **Number of Submotions (4 bytes):** The total number of submotions (bone animations) in the chunk.

For each submotion:

- **Pose Rotation (quaternion):** The pose rotation in quaternion format.

- **Bind Pose Rotation (quaternion):** The bind pose rotation in quaternion format.

- **Pose Scale Rotation (quaternion):** The pose scale rotation in quaternion format.

- **Bind Pose Scale Rotation (quaternion):** The bind pose scale rotation in quaternion format.

- **Pose Position (vec3d):** The pose position as a 3D vector.

- **Pose Scale (vec3d):** The pose scale as a 3D vector.

- **Bind Pose Position (vec3d):** The bind pose position as a 3D vector.

- **Bind Pose Scale Position (vec3d):** The bind pose scale position as a 3D vector.

- **Number of Position Keys (4 bytes):** The number of position keys in the submotion.

- **Number of Rotation Keys (4 bytes):** The number of rotation keys in the submotion.

- **Number of Scale Keys (4 bytes):** The number of scale keys in the submotion.

- **Number of Scale Rotation Keys (4 bytes):** The number of scale rotation keys in the submotion.

- **Maximum Error (4 bytes):** A floating-point value representing the maximum error for the submotion.

- **Node Name (string):** The name of the bone node associated with the submotion.

For each position key, rotation key, scale key, and scale rotation key:

- **Position Key (vec3d):** The position key as a 3D vector.

- **Time (4 bytes):** The time associated with the keyframe.

- **Rotation Key (quaternion):** The rotation key in quaternion format.

Submotions represent animations associated with specific bone nodes in the skeletal structure.

This document provides an overview of the binary structure of XSM files, with a focus on the header, chunk structure,
and key components within the file. The specific structure of an XSM file may vary depending on the application or
system that uses it.

Please note that additional details about specific chunk types and their structures can be found in the XSM file format
documentation for the relevant application or software.
