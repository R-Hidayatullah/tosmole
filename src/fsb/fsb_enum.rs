#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum FsbMode {
    Unknown = 0,
    Pcm8 = 1,
    Pcm16 = 2,
    Pcm24 = 3,
    Pcm32 = 4,
    Pcmfloat = 5,
    Gcadpcm = 6,
    Imaadpcm = 7,
    Vag = 8,
    Hevag = 9,
    Xma = 10,
    Mpeg = 11,
    Celt = 12,
    AT9 = 13,
    Xwma = 14,
    Vorbis = 15,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum FsbChunkType {
    Channels = 1,
    Frequency = 2,
    Loop = 3,
    Xmaseek = 6,
    Dspcoeff = 7,
    Xwmadata = 10,
    Vorbisdata = 11,
}
