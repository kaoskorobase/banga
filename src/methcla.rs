extern crate libc;
use libc::size_t;

pub enum Engine {}

#[link(name = "methcla")]
extern "C" {
  fn methcla_engine_free(arg: *mut Engine);
}
