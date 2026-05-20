//! # root-vortex
//!
//! Convert CERN ROOT files (TTrees) to the [Vortex](https://github.com/vortex-data/vortex)
//! columnar file format.
//!
//! ROOT files are read with the crate's internal minimal ROOT reader, without any ROOT/C++
//! dependency. Each TTree branch is mapped to a Vortex array column and the whole tree is
//! written as a Vortex `StructArray`.
//!
//! ## Supported branch types
//!
//! | ROOT C++ type          | Vortex array type   |
//! |------------------------|---------------------|
//! | `bool`                 | `PrimitiveArray<u8>`|
//! | `int8_t` / `int16_t`  | `PrimitiveArray<i8/i16>` |
//! | `int32_t`              | `PrimitiveArray<i32>` |
//! | `int64_t`              | `PrimitiveArray<i64>` |
//! | `uint8_t` / `uint16_t`| `PrimitiveArray<u8/u16>` |
//! | `uint32_t`             | `PrimitiveArray<u32>` |
//! | `uint64_t`             | `PrimitiveArray<u64>` |
//! | `float`                | `PrimitiveArray<f32>` |
//! | `double`               | `PrimitiveArray<f64>` |
//! | `std::vector<T>`       | `ListArray<T>`      |
//!
//! Other unsupported branches are skipped with a warning.
//!
//! ## Quick start
//!
//! ```no_run
//! root_vortex::convert_root_to_vortex("events.root", "events.vortex", "events").unwrap();
//! ```

mod error;
mod gen_factory;
pub mod rbase;
mod rbytes;
mod rcolors;
mod rcompress;
pub mod rcont;
mod rdict;
mod riofs;
mod rmeta;
mod root;
mod rtree;
mod rtypes;
mod rusty;
mod rvers;
mod utils;

pub mod converter;
pub mod info;

pub use error::Result;
pub use rbytes::{Unmarshaler, UnmarshalerInto};
pub use riofs::file::RootFile;
pub use rtree::branch::Branch;
pub use rtree::tree::reader::ReaderTree;
pub use rtree::tree::{StateCallBack, WriterTree};
pub use rusty::{SizedSlice, Slice};

pub use converter::convert_root_to_vortex;
pub use info::tree_info;
