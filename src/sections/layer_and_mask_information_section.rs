use crate::sections::as_u16_be;
use crate::sections::as_u32_be;
use failure::{Error, Fail};
use std::collections::HashMap;
use std::io::Cursor;
use std::io::Read;
use std::sync::mpsc::channel;

/// The LayerAndMaskInformationSection comes from the bytes in the fourth section of the PSD.
///
/// When possible we'll make the data easier to work with by storing it structures such as HashMaps.
///
/// # Note
///
/// We do not currently store all of the information that is present in the layer and mask
/// information section of the PSD. If something that you need is missing please open an issue.
///
/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// The fourth section of a Photoshop file contains information about layers and masks. This section of the document describes the formats of layer and mask records.
///
/// The complete merged image data is not stored here. The complete merged/composite image resides in the last section of the file. See See Image Data Section. If maximize compatibility is unchecked then the merged/composite is not created and the layer data must be read to reproduce the final image.
///
/// See Layer and mask information section shows the overall structure of this section. If there are no layers or masks, this section is just 4 bytes: the length field, which is set to zero. (**PSB** length is 8 bytes
///
/// 'Layr', 'Lr16' and 'Lr32' start at See Layer info. NOTE: The length of the section may already be known.)
///
/// When parsing this section pay close attention to the length of sections.
///
/// | Length   | Description                                                                                                                                                                                |
/// |----------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | 4        | Length of the layer and mask information section.<br> (**PSB** length is 8 bytes.)                                                                                                         |
/// | Variable | Layer info (see See Layer info for details).                                                                                                                                               |
/// | Variable | Global layer mask info (see See Global layer mask info for details).                                                                                                                       |
/// | Variable | (Photoshop 4.0 and later) <br> Series of tagged blocks containing various types of data. See See Additional Layer Information for the list of the types of data that can be included here. |
#[derive(Debug)]
pub struct LayerAndMaskInformationSection {
    pub(in crate) layers: HashMap<String, PsdLayer>,
}

impl LayerAndMaskInformationSection {
    /// Create a LayerAndMaskInformationSection from the bytes in the corresponding secton in a
    /// PSD file.
    pub fn from_bytes(bytes: &[u8]) -> Result<LayerAndMaskInformationSection, Error> {
        let mut cursor = Cursor::new(bytes);

        let mut two_bytes = [0; 2];
        let mut four_bytes = [0; 4];

        // The first four bytes of the section is the length marker for the layer and mask
        // information section, we won't be needing it.
        cursor.read_exact(&mut four_bytes)?;

        // Read the next four bytes to get the length of the layer info section
        cursor.read_exact(&mut four_bytes)?;
        let layer_info_section_len = as_u32_be(&four_bytes);

        // Next 2 bytes is the layer count
        cursor.read_exact(&mut two_bytes)?;
        let layer_count = as_u16_be(&two_bytes);

        // Read each layer
        for layer_num in 0..layer_count {
            let layer = read_layer_record(bytes, &mut cursor)?;
        }

        // inside of read_layer method skip over data that we don't need right now, but
        // leave a comment

        unimplemented!();
    }
}

/// Read bytes, starting from the cursor, until we've processed all of the data for a layer in
/// the layer records section.
///
/// At the moment we skip over some of the data.
///
/// Please open an issue if there is data in here that you need that we don't currently parse.
///
/// # [Adobe Docs](https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/)
///
/// Information about each layer.
///
/// | Length                 | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
/// |------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | 4 * 4                  | Rectangle containing the contents of the layer. Specified as top, left, bottom, right coordinates                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
/// | 2                      | Number of channels in the layer                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
/// | 6 * number of channels | Channel information. Six bytes per channel,<br> consisting of: 2 bytes for Channel ID: 0 = red, 1 = green, etc.; <br> -1 = transparency mask; -2 = user supplied layer mask, -3 real user supplied layer mask (when both a user mask and a vector mask are present) <br>  4 bytes for length of corresponding channel data. (**PSB** 8 bytes for length of corresponding channel data.) See See Channel image data for structure of channel data.                                                                                                                                                 |
/// | 4                      | Blend mode signature: '8BIM'                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
/// | 4                      | Blend mode key: <br> 'pass' = pass through, 'norm' = normal, 'diss' = dissolve, 'dark' = darken, 'mul ' = multiply, 'idiv' = color burn, 'lbrn' = linear burn, 'dkCl' = darker color, 'lite' = lighten, 'scrn' = screen, 'div ' = color dodge, 'lddg' = linear dodge, 'lgCl' = lighter color, 'over' = overlay, 'sLit' = soft light, 'hLit' = hard light, 'vLit' = vivid light, 'lLit' = linear light, 'pLit' = pin light, 'hMix' = hard mix, 'diff' = difference, 'smud' = exclusion, 'fsub' = subtract, 'fdiv' = divide 'hue ' = hue, 'sat ' = saturation, 'colr' = color, 'lum ' = luminosity, |
/// | 1                      | Opacity. 0 = transparent ... 255 = opaque                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
/// | 1                      | Clipping: 0 = base, 1 = non-base                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
/// | 1                      | Flags: <br> bit 0 = transparency protected; <br> bit 1 = visible; <br> bit 2 = obsolete; <br> bit 3 = 1 for Photoshop 5.0 and later, tells if bit 4 has useful information; <br> bit 4 = pixel data irrelevant to appearance of document                                                                                                                                                                                                                                                                                                                                                          |
/// | 1                      | Filler (zero)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
/// | 4                      | Length of the extra data field ( = the total length of the next five fields).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
/// | Variable               | Layer mask data: See See Layer mask / adjustment layer data for structure. Can be 40 bytes, 24 bytes, or 4 bytes if no layer mask.                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
/// | Variable               | Layer blending ranges: See See Layer blending ranges data.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
/// | Variable               | Layer name: Pascal string, padded to a multiple of 4 bytes.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
fn read_layer_record(bytes: &[u8], cursor: &mut Cursor<&[u8]>) -> Result<LayerRecord, Error> {
    let mut channels = vec![];

    // TODO: Create a PsdCursor that provides an easy API for skipping bytes.
    // cursor.skip(2)
    // let two_bytes: &[u8] = cursor.read(2) (advances cursor by 2)
    let mut one_byte = [0; 1];
    let mut two_bytes = [0; 2];
    let mut four_bytes = [0; 4];
    let mut six_bytes = [0; 6];

    // We do not currently parse the layer rectangle, skip it
    let rectangle_bytes = 16;
    cursor.set_position(cursor.position() + rectangle_bytes);

    // Get the number of channels in the layer
    cursor.read(&mut two_bytes)?;
    let channel_count = as_u16_be(&two_bytes);

    // Read the channel information
    for _ in 0..channel_count {
        cursor.read_exact(&mut six_bytes);
        let channel_id = six_bytes[1] as i8;
        let channel_id = PsdLayerChannel::new(channel_id)?;

        let channel_length = as_u32_be(&[six_bytes[2], six_bytes[3], six_bytes[4], six_bytes[5]]);

        channels.push((channel_id, channel_length));
    }

    // We do not currently parse the blend mode signature, skip it
    cursor.read_exact(&mut four_bytes)?;

    // We do not currently parse the blend mode key, skip it
    cursor.read_exact(&mut four_bytes)?;

    // We do not currently parse the opacity, skip it
    cursor.read_exact(&mut one_byte)?;

    // We do not currently parse the clipping, skip it
    cursor.read_exact(&mut one_byte)?;

    // We do not currently parse the flags, skip it
    cursor.read_exact(&mut one_byte)?;

    // We do not currently parse the filter, skip it
    cursor.read_exact(&mut one_byte)?;

    // We do not currently use the length of the extra data field, skip it
    cursor.read_exact(&mut four_bytes)?;

    let name = "".to_string();

    Ok(LayerRecord { name, channels })
}

/// A layer record within the layer info section
#[derive(Debug)]
struct LayerRecord {
    /// The name of the layer
    name: String,
    /// The channels that this record has and the number of bytes in each channel.
    channels: Vec<(PsdLayerChannel, u32)>,
}

/// Information about a layer in a PSD file.
#[derive(Debug)]
pub struct PsdLayer {
    /// The channels of the layer, stored separately.
    ///
    /// You can combine these channels into a final image. For example, you might combine
    /// the Red, Green and Blue channels, or you might also combine the TransparencyMask (alpha)
    /// channel, or you might make use of the layer masks.
    ///
    /// Storing the channels separately allows for this flexability.
    channels: HashMap<PsdLayerChannel, Vec<u8>>,
}

/// The different kinds of channels in a layer (red, green, blue, ...).
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[allow(missing_docs)]
enum PsdLayerChannel {
    Red = 0,
    Green = 1,
    Blue = 2,
    TransparencyMask = -1,
    UserSuppliedLayerMask = -2,
    RealUserSuppliedLayerMask = -3,
}

/// Represents an invalid layer channel id
#[derive(Debug, Fail)]
pub enum PsdLayerChannelError {
    #[fail(
        display = "{} is an invalid channel id, must be 0, 1, 2, -1, -2, or -3.",
        channel_id
    )]
    InvalidChannel { channel_id: i8 },
}

impl PsdLayerChannel {
    /// Create a new PsdLayerChannel
    pub fn new(channel_id: i8) -> Result<PsdLayerChannel, Error> {
        match channel_id {
            0 => Ok(PsdLayerChannel::Red),
            1 => Ok(PsdLayerChannel::Green),
            2 => Ok(PsdLayerChannel::Blue),
            -1 => Ok(PsdLayerChannel::TransparencyMask),
            -2 => Ok(PsdLayerChannel::UserSuppliedLayerMask),
            -3 => Ok(PsdLayerChannel::RealUserSuppliedLayerMask),
            _ => Err(PsdLayerChannelError::InvalidChannel { channel_id })?,
        }
    }
}
