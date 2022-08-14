#![forbid(unsafe_code)]

use crate::handle::{DirHandle, FileHandle, Handle};
use std::fs;
use std::io::Read;

type Callback<'a> = dyn FnMut(&mut Handle) + 'a;

#[derive(Default)]
pub struct Walker<'a> {
    callbacks: Vec<Box<Callback<'a>>>,
}

impl<'a> Walker<'a> {
    pub fn new() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }

    pub fn add_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&mut Handle) + 'a,
    {
        self.callbacks.push(Box::new(callback));
    }

    pub fn walk<P: AsRef<std::path::Path>>(mut self, path: P) -> std::io::Result<()> {
        if self.callbacks.is_empty() {
            return Ok(());
        }
        let mut indices = Vec::new();
        for i in 0..self.callbacks.len() {
            indices.push(i);
        }
        self.walk_private(path, &indices)
    }

    fn walk_private<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        indices: &[usize],
    ) -> std::io::Result<()> {
        let metadata = fs::metadata(path.as_ref())?;
        let file_type = metadata.file_type();
        let mut walk_result: std::io::Result<()> = Ok(());
        if file_type.is_dir() {
            let mut new_indices = Vec::<usize>::new();
            for i in indices {
                let mut dir_handle = Handle::Dir(DirHandle::new(path.as_ref()));
                self.callbacks[*i](&mut dir_handle);
                if let Handle::Dir(x) = &dir_handle {
                    if x.is_called() {
                        new_indices.push(*i);
                    }
                };
            }

            if new_indices.is_empty() {
                return Ok(());
            }

            let paths = fs::read_dir(path.as_ref()).unwrap();
            for new_path in paths {
                let new_result = self.walk_private(new_path.unwrap().path(), &new_indices);
                if walk_result.is_ok() {
                    walk_result = new_result;
                }
            }
        } else if file_type.is_file() {
            let mut new_indices = Vec::<usize>::new();
            for i in indices {
                let mut file_handle = Handle::File(FileHandle::new(path.as_ref()));
                self.callbacks[*i](&mut file_handle);
                if let Handle::File(x) = &file_handle {
                    if x.is_called() {
                        new_indices.push(*i);
                    }
                };
            }

            if new_indices.is_empty() {
                return Ok(());
            }

            let mut file = fs::File::open(path.as_ref()).expect("no file found");
            let mut buffer = vec![0; metadata.len() as usize];
            let _size = file.read(&mut buffer);
            let content = buffer.as_slice();
            let mut content_handle = Handle::Content {
                file_path: path.as_ref(),
                content,
            };
            for i in new_indices {
                self.callbacks[i](&mut content_handle);
            }
        }
        walk_result
    }
}
