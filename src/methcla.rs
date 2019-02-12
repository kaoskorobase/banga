#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/methcla_bindings.rs"));

// extern crate libc;
// use libc::size_t;
//
// pub enum Engine {}
//
// #[link(name = "methcla")]
// extern "C" {
//   fn methcla_engine_free(arg: *mut Engine);
// }
