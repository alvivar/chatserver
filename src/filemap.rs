use std::{collections::HashMap, sync::Arc};

pub enum FileData {
    Str(&'static str),
    Bytes(&'static [u8]),
}

pub struct FileMap {
    pub data: FileData,
    pub mime_type: &'static str,
}

impl FileMap {
    pub fn static_files() -> Arc<HashMap<&'static str, FileMap>> {
        let filemap: HashMap<&str, FileMap> = [
            (
                "/index.html",
                FileMap {
                    data: FileData::Str(include_str!("../web/index.html")),
                    mime_type: "text/html",
                },
            ),
            (
                "/main.js",
                FileMap {
                    data: FileData::Str(include_str!("../web/main.js")),
                    mime_type: "application/javascript",
                },
            ),
            (
                "/style.css",
                FileMap {
                    data: FileData::Str(include_str!("../web/style.css")),
                    mime_type: "text/css",
                },
            ),
            (
                "/favicon.ico",
                FileMap {
                    data: FileData::Bytes(include_bytes!("../web/favicon.ico")),
                    mime_type: "image/x-icon",
                },
            ),
        ]
        .into();

        Arc::new(filemap)
    }
}
