#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "stb_image_write.h"
#include <stdlib.h>
#include <string.h>

// Wrapper for stbi_load_from_memory
unsigned char *stb_load_from_memory(const unsigned char *buffer, int len,
                                    int *x, int *y, int *channels)
{
    return stbi_load_from_memory(buffer, len, x, y, channels, 0);
}

// Free image
void stb_free_image(unsigned char *data)
{
    stbi_image_free(data);
}

// Memory buffer for writing PNG
typedef struct
{
    unsigned char *data;
    size_t size;
    size_t capacity;
} MemBuffer;

static void png_write_callback(void *context, void *data, int size)
{
    MemBuffer *buf = (MemBuffer *)context;
    if (buf->size + size > buf->capacity)
    {
        buf->capacity = (buf->capacity + size) * 2;
        buf->data = (unsigned char *)realloc(buf->data, buf->capacity);
    }
    memcpy(buf->data + buf->size, data, size);
    buf->size += size;
}

// Encode PNG into memory
unsigned char *stb_write_png_mem(const unsigned char *pixels,
                                 int w, int h, int comp,
                                 int *out_len)
{
    MemBuffer buf;
    buf.data = (unsigned char *)malloc(1024);
    buf.size = 0;
    buf.capacity = 1024;

    if (!stbi_write_png_to_func(png_write_callback, &buf,
                                w, h, comp, pixels, w * comp))
    {
        free(buf.data);
        *out_len = 0;
        return NULL;
    }

    *out_len = (int)buf.size;
    return buf.data; // caller must free
}
