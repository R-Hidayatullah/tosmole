# IES File Structure

The **IES file format** is used in various applications, including the game "Tree of Savior," to store tabular data.
This document explains the binary structure of IES files, with an emphasis on the header and its key components.

## IES Header

An IES file begins with a header that contains essential information about the structure and content of the file.

### Header Fields

- **Name (128 bytes):** The name field specifies the name of the IES file. It is typically a null-terminated string,
  meaning it's trimmed to remove any trailing null characters.

- **Padding (4 bytes):** This field is reserved for padding and is generally ignored during parsing.

- **Data Offset (4 bytes):** It indicates the offset to the data section within the IES file.

- **Resource Offset (4 bytes):** The resource offset specifies the offset to the resource section within the IES file.

- **File Size (4 bytes):** This field provides the total size of the IES file in bytes.

- **Padding (2 bytes):** Another padding field.

- **Row Count (2 bytes):** It specifies the total number of rows in the data section of the IES file.

- **Column Count (2 bytes):** Indicates the total number of columns in the tabular data.

- **Number Column Count (2 bytes):** The number of columns that store numerical data.

- **String Column Count (2 bytes):** The count of columns that store string data.

- **Padding (2 bytes):** An additional padding field.

## Column Definitions

Following the header, an IES file contains a series of column definitions. The number of column definitions is
determined by the "Column Count" field in the header.

### Column Structure

Each column definition consists of the following components:

- **Name (64 bytes):** The name of the column, typically representing the column's label or identifier.

- **Second Name (64 bytes):** A secondary name for the column, if applicable.

- **Column Type (2 bytes):** A numeric code that signifies the data type of the column. The codes generally represent:
    - 0: Float
    - 1: String
    - 2: Second String

- **Padding (4 bytes):** Reserved for padding.

- **Position (2 bytes):** The position of the column within the tabular data.

## Tabular Data (Rows)

After the column definitions, the IES file stores tabular data. Each row represents a data entry and consists of values
corresponding to the columns defined earlier. The structure of each row is dependent on the data type of the respective
column.

- **Padding (4 bytes):** A padding field that is generally ignored during parsing.

- **Count (2 bytes):** Indicates the number of bytes in the row for variable-length data (e.g., strings).

- **Data (variable length):** The actual data for the row, which may include numerical values, strings, or other
  relevant information.

Please note that the structure of the rows may vary depending on the specific IES file's use case.

This document provides an overview of the binary structure of IES files and their header components. The specific
structure of an IES file may differ based on the application or system that uses it.