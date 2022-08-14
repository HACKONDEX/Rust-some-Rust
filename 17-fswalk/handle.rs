#![forbid(unsafe_code)]

use std::path::Path;

pub enum Handle<'a> {
    Dir(DirHandle<'a>),
    File(FileHandle<'a>),
    Content {
        file_path: &'a Path,
        content: &'a [u8],
    },
}

pub struct DirHandle<'a> {
    path: &'a Path,
    descend_called: bool,
}

impl<'a> DirHandle<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self {
            path,
            descend_called: false,
        }
    }

    pub fn descend(&mut self) {
        self.descend_called = true;
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn reset(&mut self) {
        self.descend_called = false;
    }

    pub fn is_called(&self) -> bool {
        self.descend_called
    }
}

pub struct FileHandle<'a> {
    path: &'a Path,
    read_called: bool,
}

impl<'a> FileHandle<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self {
            path,
            read_called: false,
        }
    }

    pub fn read(&mut self) {
        self.read_called = true;
    }

    pub fn path(&self) -> &Path {
        self.path
    }

    pub fn reset(&mut self) {
        self.read_called = false;
    }

    pub fn is_called(&self) -> bool {
        self.read_called
    }
}
