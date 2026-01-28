use anyhow::{Result, bail};
use gtk::{
    gdk_pixbuf::{Pixbuf, PixbufFormat},
    gio::{
        self, Cancellable, FILE_ATTRIBUTE_STANDARD_CONTENT_TYPE, FileQueryInfoFlags,
        MemoryInputStream, prelude::FileExt,
    },
    glib,
};
use std::path::PathBuf;

pub struct Icon {
    pub pixbuf: Pixbuf,
}
impl Icon {
    const SCALE_HEIGHT: i32 = 512;

    pub fn from_path(path: &PathBuf) -> Result<Icon> {
        let file = gio::File::for_path(path);

        let pixbuf = if let Some(pixbuf_format) = Self::get_pixbuf_format_from_file(&file)
            && pixbuf_format.is_scalable()
        {
            Pixbuf::from_file_at_scale(path, -1, Self::SCALE_HEIGHT, true)
        } else {
            Pixbuf::from_file(path)
        };

        let pixbuf = match pixbuf {
            Err(error) => {
                bail!("Could not load image into a Pixbuf: '{error:?}'");
            }
            Ok(pixbuf) => pixbuf,
        };

        Ok(Self { pixbuf })
    }

    pub fn from_bytes(bytes: &Vec<u8>, mimetype: Option<String>) -> Result<Icon> {
        let pixbuf_format =
            mimetype.and_then(|mimetype| Self::get_pixbuf_format_from_mimetype(&mimetype));
        let g_bytes = glib::Bytes::from(bytes);
        let stream = MemoryInputStream::from_bytes(&g_bytes);

        let pixbuf = if let Some(pixbuf_format) = pixbuf_format
            && pixbuf_format.is_scalable()
        {
            Pixbuf::from_stream_at_scale(&stream, -1, Self::SCALE_HEIGHT, true, Cancellable::NONE)
        } else {
            Pixbuf::from_stream(&stream, Cancellable::NONE)
        };

        let pixbuf = match pixbuf {
            Err(error) => {
                bail!("Could not load image stream into a Pixbuf: '{error:?}'");
            }
            Ok(pixbuf) => pixbuf,
        };

        Ok(Self { pixbuf })
    }

    fn get_pixbuf_format_from_mimetype(mimetype: &str) -> Option<PixbufFormat> {
        Pixbuf::formats()
            .into_iter()
            .find(|format| format.mime_types().iter().any(|mtype| *mtype == mimetype))
    }

    fn get_pixbuf_format_from_file(file: &gio::File) -> Option<PixbufFormat> {
        file.query_info(
            FILE_ATTRIBUTE_STANDARD_CONTENT_TYPE,
            FileQueryInfoFlags::NONE,
            None::<&Cancellable>,
        )
        .ok()
        .and_then(|file_info| file_info.content_type())
        .and_then(|mimetype| Self::get_pixbuf_format_from_mimetype(&mimetype))
    }
}
