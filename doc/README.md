# Tree of Savior File Types

This README provides an overview of file types used in the game "Tree of Savior" and their respective structures. The
game uses various file formats to store different types of data.

## IES

**IES** files are used to store table-like data structures that save game-related data. They are commonly employed to
hold information such as item data, character statistics, and other game-related attributes.

### Binary Structure

IES files generally have a structured format with data organized into rows and columns. Specific details about the
structure may vary depending on the type of data stored, but typically follow a tabular format.

## XAC

**XAC** files are used to store 3D model data, including models and a list of textures used in those models. These files
play a crucial role in rendering game assets.

### Binary Structure

XAC files contain data related to 3D models, including vertex information, texture coordinates, and links to textures.
The structure may differ depending on the complexity of the 3D model.

## XSM

**XSM** files contain animation data. Each XSM file typically represents a single animation, such as "idle," "attack,"
or "walk." These files are associated with XAC files to animate 3D models.

### Binary Structure

The binary structure of XSM files contains data required for animating 3D models, including keyframes, bone
transformations, and animation duration.

## IPF

**IPF** files serve as archive files in the game. They act as containers for various types of data used in "Tree of
Savior." IPF files are notable for being encrypted using CRC32 and compressed with the FLATE algorithm.

### Binary Structure

- **Structure for Data Files:** Data files within the IPF container start with data structures specific to their file
  types. These data structures may be encrypted and compressed.
- **Header for Each File:** Each data file within the IPF container contains a header with information relevant to that
  file.
- **Header for IPF File:** The IPF file begins with a header that provides essential information about the archive.

## Endianness

All binary files used in "Tree of Savior" are parsed using the little-endian byte order, which is a common byte order
for x86-based systems.

This README provides a high-level overview of the file types and their binary structures used in the game "Tree of
Savior." For in-depth technical details and specifics, consult the game's documentation or resources related to game
file formats.
