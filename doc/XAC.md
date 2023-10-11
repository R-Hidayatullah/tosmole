# XAC File Structure

The **XAC file format** is used in various applications, including game development, to store model composition data.
This document explains the binary structure of XAC files, with an emphasis on the header and its key components.

## XAC Header

An XAC file begins with a header that contains essential information about the structure and content of the file.

### Header Fields

- **Magic Number (4 bytes):** A unique identifier or magic number that signifies the file type. It should be `XAC ` for
  a valid XAC file.

- **Major Version (1 byte):** The major version of the XAC file format.

- **Minor Version (1 byte):** The minor version of the XAC file format.

- **Big Endian (1 byte):** A boolean flag indicating whether the file is in big-endian format (usually false for
  little-endian systems).

- **Multiply Order (1 byte):** An order used for multiplication.

### Header Validation

- The magic number should be "XAC " to be considered a valid XAC file.
- The major and minor version should match the expected version (e.g., v1.0).

## Chunk Structure

Following the header, an XAC file contains a series of chunks, each with a specific type, length, and version. Chunks
represent various components of the model composition.

### Chunk Components

Each chunk consists of the following components:

- **Type ID (4 bytes):** An identifier that indicates the type of the chunk. It corresponds to specific components like
  mesh, skinning, material definitions, and more.

- **Length (4 bytes):** The length of the chunk, specifying the size of the data contained within the chunk.

- **Version (4 bytes):** The version of the chunk format.

### Chunk Processing

The XAC file parser processes each chunk based on its type ID, reading and interpreting the data within the chunk.

## Model Composition Data

The actual model composition data is stored within the chunks. The structure of the model composition data may vary
based on the type of chunk. Common components within the model composition data include:

- **Vertices and Attributes:** Information related to the vertices and their attributes, such as position, normals,
  tangents, UV coordinates, colors, and influence ranges.

- **Materials:** Definitions of materials used in the composition, including properties like ambient color, diffuse
  color, specular color, emissive color, shininess, opacity, and more.

- **Shader Materials:** Definitions of shader materials with integer, float, boolean, and string properties.

- **Node Hierarchy:** Information about the hierarchy of nodes, including their transformations and parent-child
  relationships.

- **Meshes:** Information about individual meshes, including vertices, indices, and sub-meshes.

- **Skinning:** Data related to mesh skinning, including local bones and influence data.

- **Metadata:** General information about the XAC file, including exporter details, original file name, export date, and
  more.

This document provides an overview of the binary structure of XAC files, with an emphasis on the header, chunk
structure, and key components within the file. The specific structure of an XAC file may vary depending on the
application or system that uses it.

Please note that additional details about specific chunk types and their structures can be found in the XAC file format
documentation for the relevant application or software.
