use std::path::PathBuf;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs::File;
use std::io::Write;
use std::io::BufWriter;
use std::io;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock, MutexGuard};
use std::fmt;
use std::ops::Deref;
use std::cell::RefCell;
use core::error::*;
use rand::{thread_rng, Rng};
// use memmap::{Mmap, Protection};
use fst::raw::MmapReadOnly;

#[derive(Clone, Debug)]
pub struct SegmentId(pub String);

pub fn generate_segment_name() -> SegmentId {
    static CHARS: &'static [u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let random_name: String = (0..8)
            .map(|_| thread_rng().choose(CHARS).unwrap().clone() as char)
            .collect();
    SegmentId( String::from("_") + &random_name)
}


#[derive(Clone)]
pub struct Directory {
    index_path: PathBuf,
    mmap_cache: Arc<Mutex<HashMap<PathBuf, MmapReadOnly>>>,
}

impl fmt::Debug for Directory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       write!(f, "Directory({:?})", self.index_path)
   }
}

fn open_mmap(full_path: &PathBuf) -> Result<MmapReadOnly> {
    match MmapReadOnly::open_path(full_path.clone()) {
        Ok(mmapped_file) => Ok(mmapped_file),
        Err(ioerr) => {
            // TODO add file
            let error_msg = format!("Read-Only MMap of {:?} failed", full_path);
            return Err(Error::IOError(ioerr.kind(), error_msg));
        }
    }
}

impl Directory {

    pub fn from(filepath: &str) -> Directory {
        Directory {
            index_path: PathBuf::from(filepath),
            mmap_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn resolve_path(&self, relative_path: &PathBuf) -> PathBuf {
        self.index_path.join(relative_path)
    }

    pub fn segment(&self, segment_id: &SegmentId) -> Segment {
        Segment {
            directory: self.clone(),
            segment_id: segment_id.clone()
        }
    }

    pub fn new_segment(&self,) -> Segment {
        // TODO check it does not exists
        self.segment(&generate_segment_name())
    }

    fn open_writable(&self, relative_path: &PathBuf) -> Result<File> {
        let full_path = self.resolve_path(relative_path);
        match File::create(full_path.clone()) {
            Ok(f) => Ok(f),
            Err(err) => {
                let path_str = full_path.to_str().unwrap_or("<error on to_str>");
                return Err(Error::IOError(err.kind(), String::from("Could not create file") + path_str))
            }
        }
    }

    fn mmap(&self, relative_path: &PathBuf) -> Result<MmapReadOnly> {
        let full_path = self.resolve_path(relative_path);
        let mut cache_mutex = self.mmap_cache.deref();
        match cache_mutex.lock() {
            Ok(mut cache) => {
                if !cache.contains_key(&full_path) {
                    cache.insert(full_path.clone(), try!(open_mmap(&full_path)) );
                }
                let mmap_readonly: &MmapReadOnly = cache.get(&full_path).unwrap();
                // TODO remove if a proper clone is available
                let len = unsafe { mmap_readonly.as_slice().len() };
                return Ok(mmap_readonly.range(0, len))
            },
            Err(_) => {
                return Err(Error::CannotAcquireLock(String::from("Cannot acquire mmap cache lock.")))
            }
        }
    }
}

/////////////////////////
// Segment

pub enum SegmentComponent {
    POSTINGS,
    POSITIONS,
    TERMS,
}

#[derive(Debug)]
pub struct Segment {
    directory: Directory,
    segment_id: SegmentId,
}

impl Segment {
    fn path_suffix(component: SegmentComponent)-> &'static str {
        match component {
            SegmentComponent::POSTINGS => ".idx",
            SegmentComponent::POSITIONS => ".pos",
            SegmentComponent::TERMS => ".term",
        }
    }

    fn relative_path(&self, component: SegmentComponent) -> PathBuf {
        let SegmentId(ref segment_id_str) = self.segment_id;
        let filename = String::new() + segment_id_str + Segment::path_suffix(component);
        PathBuf::from(filename)
    }

    pub fn mmap(&self, component: SegmentComponent) -> Result<MmapReadOnly> {
        let path = self.relative_path(component);
        self.directory.mmap(&path)
    }

    pub fn open_writable(&self, component: SegmentComponent) -> Result<File> {
        let path = self.relative_path(component);
        self.directory.open_writable(&path)
    }
}
