#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/methcla_bindings.rs"));

use fixedbitset::FixedBitSet;
use rosc::{OscBundle, OscMessage, OscPacket, OscType};
use std::ffi::{c_void, CStr};
use std::path::PathBuf;
use std::ptr;

struct Allocator {
    ids: FixedBitSet,
}

impl Allocator {
    pub fn with_capacity(size: usize) -> Self {
        Allocator {
            ids: FixedBitSet::with_capacity(size),
        }
    }
    pub fn alloc(self: &mut Self) -> i32 {
        let mut r = None;
        for i in 0..self.ids.len() - 1 {
            if !self.ids[i] {
                self.ids.set(i, true);
                r = Some(i as i32);
                break;
            }
        }
        match r {
            Some(x) => x,
            None => panic!("Couldn't find free node id"),
        }
    }
    pub fn free(self: &mut Self, id: i32) {
        if id >= 0 && (id as usize) < self.ids.len() {
            self.ids.set(id as usize, false);
        }
    }
}

#[derive(Debug)]
pub struct NodeId(i32);

pub trait Node {
    fn node_id(&self) -> NodeId;
}

pub struct Group(i32);

impl Group {
    pub fn from_i32(id: i32) -> Self {
        Group(id)
    }
}

impl Node for Group {
    fn node_id(&self) -> NodeId {
        NodeId(self.0)
    }
}

pub struct Synth(i32);

impl Synth {
    pub fn from_i32(id: i32) -> Self {
        Synth(id)
    }
}

impl Node for Synth {
    fn node_id(&self) -> NodeId {
        NodeId(self.0)
    }
}

pub struct AudioBus(i32);

impl AudioBus {
    pub fn from_i32(id: i32) -> Self {
        AudioBus(id)
    }

    pub fn bus_id(self: &Self) -> i32 {
        self.0
    }
}

#[derive(Debug)]
pub struct NodePlacement {
    target: NodeId,
    placement: Methcla_NodePlacement,
}

impl NodePlacement {
    pub fn head(group: &Group) -> Self {
        NodePlacement {
            target: group.node_id(),
            placement: kMethcla_NodePlacementHeadOfGroup,
        }
    }
    pub fn tail(group: &Group) -> Self {
        NodePlacement {
            target: group.node_id(),
            placement: kMethcla_NodePlacementTailOfGroup,
        }
    }
    pub fn before(node: &Node) -> Self {
        NodePlacement {
            target: node.node_id(),
            placement: kMethcla_NodePlacementBeforeNode,
        }
    }
    pub fn after(node: &Node) -> Self {
        NodePlacement {
            target: node.node_id(),
            placement: kMethcla_NodePlacementBeforeNode,
        }
    }
}

#[derive(Debug)]
pub struct Error {
    code: Methcla_ErrorCode,
    message: String,
}

impl Error {
    fn to_result<a>(result: a, e: &Methcla_Error) -> Result<a, Self> {
        if e.error_code == kMethcla_NoError {
            Ok(result)
        } else {
            Err(Error {
                code: e.error_code,
                message: unsafe {
                    CStr::from_ptr(e.error_message)
                        .to_string_lossy()
                        .into_owned()
                },
            })
        }
    }
    fn check(error: &Methcla_Error) -> Result<(), Self> {
        Self::to_result((), error)
    }
}

struct EngineOptions {
    ptr: *mut Methcla_EngineOptions,
}

impl EngineOptions {
    fn new(options: &Options) -> Result<Self, Error> {
        let mut ptr = ptr::null_mut();
        let mut addr = &mut ptr as *mut *mut Methcla_EngineOptions;
        unsafe {
            Error::check(&methcla_engine_options_new(addr))?;
            methcla_engine_options_set_sample_rate(ptr, options.sample_rate);
            methcla_engine_options_set_block_size(ptr, options.block_size);
            methcla_engine_options_set_realtime_memory_size(ptr, options.realtime_memory_size);
            methcla_engine_options_set_max_num_nodes(ptr, options.max_num_nodes);
            methcla_engine_options_set_max_num_audio_buses(ptr, options.max_num_audio_buses);
            Ok(EngineOptions { ptr: ptr })
        }
    }
}

impl Drop for EngineOptions {
    fn drop(&mut self) {
        unsafe {
            methcla_engine_options_free(self.ptr);
        }
    }
}

struct AudioDriverOptions {
    ptr: *mut Methcla_AudioDriverOptions,
}

impl AudioDriverOptions {
    fn new(options: &Options) -> Result<Self, Error> {
        let mut ptr = ptr::null_mut();
        let mut addr = &mut ptr as *mut *mut Methcla_AudioDriverOptions;
        unsafe {
            Error::check(&methcla_audio_driver_options_new(addr))?;
            methcla_audio_driver_options_set_sample_rate(ptr, options.sample_rate);
            methcla_audio_driver_options_set_buffer_size(ptr, options.block_size);
            methcla_audio_driver_options_set_num_inputs(ptr, options.num_hardware_inputs);
            methcla_audio_driver_options_set_num_outputs(ptr, options.num_hardware_outputs);
            Ok(AudioDriverOptions { ptr: ptr })
        }
    }
}

impl Drop for AudioDriverOptions {
    fn drop(&mut self) {
        unsafe {
            methcla_audio_driver_options_free(self.ptr);
        }
    }
}

// struct Resource<A, F: Fn(*mut A) -> ()> {
//     ptr: *mut A,
//     destructor: F,
// }
//
// impl<A, F: Fn(*mut A) -> ()> Resource<A, F> {
//     fn new(destructor: unsafe extern "C" fn(*mut A) -> ()) -> Resource<A, impl Fn(*mut A) -> ()> {
//         Resource {
//             ptr: ptr::null_mut(),
//             destructor: move |a: *mut A| -> () { unsafe { destructor(a) } },
//         }
//     }
//
//     fn addr(self: &Self) -> *mut *mut A {
//         &mut self.ptr as *mut *mut A
//     }
// }
//
// impl<A, F: Fn(*mut A) -> ()> Drop for Resource<A, F> {
//     fn drop(&mut self) {
//         unsafe {
//             (self.destructor)(self.ptr);
//         }
//     }
// }

pub struct Options {
    sample_rate: usize,
    block_size: usize,
    realtime_memory_size: usize,
    max_num_nodes: usize,
    max_num_audio_buses: usize,
    num_hardware_inputs: usize,
    num_hardware_outputs: usize,
    plugin_directories: Vec<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            sample_rate: 44100,
            block_size: 512,
            realtime_memory_size: 1024 * 1024,
            max_num_nodes: 1024,
            max_num_audio_buses: 1024,
            num_hardware_inputs: 2,
            num_hardware_outputs: 2,
            plugin_directories: vec![],
        }
    }
}

pub struct Engine {
    engine: *mut Methcla_Engine,
    nodeIdAllocator: Allocator,
    audioBusIdAllocator: Allocator,
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            methcla_engine_free(self.engine);
        }
    }
}

impl Engine {
    pub fn new(options: &Options) -> Result<Self, Error> {
        let mut engine_options = EngineOptions::new(options)?;
        let mut audio_driver_options = AudioDriverOptions::new(options)?;
        let mut audio_driver: *mut Methcla_AudioDriver = ptr::null_mut();
        let audio_driver_ptr = &mut audio_driver as *mut *mut Methcla_AudioDriver;
        let mut engine: *mut Methcla_Engine = ptr::null_mut();
        let engine_ptr = &mut engine as *mut *mut Methcla_Engine;
        unsafe {
            Error::check(&methcla_default_audio_driver(
                audio_driver_options.ptr,
                audio_driver_ptr,
            ))?;
            Error::check(&methcla_engine_new_with_driver(
                engine_options.ptr,
                audio_driver,
                engine_ptr,
            ))?;
        }
        Ok(Engine {
            engine: engine,
            nodeIdAllocator: Allocator::with_capacity(1000),
            audioBusIdAllocator: Allocator::with_capacity(1000),
        })
    }

    pub fn send(self: &Self, bytes: &[u8]) -> Result<(), Error> {
        unsafe {
            let packet = Methcla_OSCPacket {
                data: bytes.as_ptr() as *const c_void,
                size: bytes.len(),
            };
            Error::check(&methcla_engine_send(
                self.engine,
                &packet as *const Methcla_OSCPacket,
            ))
        }
    }
}

pub type Time = f64;

pub struct Request<'a> {
    node_id_allocator: &'a mut Allocator,
    audio_bus_id_allocator: &'a mut Allocator,
    bundle: OscBundle,
}

impl<'a> Request<'a> {
    fn at(
        time: Time,
        node_id_allocator: &'a mut Allocator,
        audio_bus_id_allocator: &'a mut Allocator,
    ) -> Self {
        unsafe {
            let timetag = methcla_time_to_uint64(time);
            let sec = (timetag >> 32) as u32;
            let frac = (timetag & 0xFFFF) as u32;
            Request {
                bundle: OscBundle {
                    timetag: OscType::Time(sec, frac),
                    content: Vec::new(),
                },
                node_id_allocator: node_id_allocator,
                audio_bus_id_allocator: audio_bus_id_allocator,
            }
        }
    }

    fn add_msg(self: &mut Self, message: OscMessage) {
        self.bundle.content.push(OscPacket::Message(message))
    }

    pub fn group(self: &mut Self, node_placement: &NodePlacement) -> Group {
        let id = self.node_id_allocator.alloc();
        self.add_msg(OscMessage {
            addr: "/group/new".to_string(),
            args: Some(vec![
                OscType::Int(id),
                OscType::Int(node_placement.target.0),
                OscType::Int(node_placement.placement as i32),
            ]),
        });
        Group::from_i32(id)
    }

    pub fn free_all(self: &mut Self, group: &Group) {
        self.add_msg(OscMessage {
            addr: "/group/freeAll".to_string(),
            args: Some(vec![OscType::Int(group.node_id().0)]),
        })
    }

    pub fn synth(
        self: &mut Self,
        synthdef: &str,
        node_placement: &NodePlacement,
        controls: &[f32],
        options: Option<Vec<OscType>>,
    ) -> Synth {
        let id = self.node_id_allocator.alloc();
        let mut args = vec![
            OscType::String(synthdef.to_string()),
            OscType::Int(id),
            OscType::Int(node_placement.target.0),
            OscType::Int(node_placement.placement as i32),
        ];
        for x in controls {
            args.push(OscType::Float(*x));
        }
        // TODO: Serialize options as OSC array (support currently missing in rosc)
        self.add_msg(OscMessage {
            addr: "/synth/new".to_string(),
            args: Some(args),
        });
        Synth::from_i32(id)
    }

    pub fn activate(self: &mut Self, synth: &Synth) {
        self.add_msg(OscMessage {
            addr: "/synth/activate".to_string(),
            args: Some(vec![OscType::Int(synth.node_id().0)]),
        })
    }

    pub fn map_input(
        self: &mut Self,
        synth: &Synth,
        index: usize,
        bus: &AudioBus,
        flags: Methcla_BusMappingFlags,
    ) {
        self.add_msg(OscMessage {
            addr: "/synth/map/input".to_string(),
            args: Some(vec![
                OscType::Int(synth.node_id().0),
                OscType::Int(index as i32),
                OscType::Int(bus.bus_id()),
                OscType::Int(flags.0 as i32),
            ]),
        })
    }

    pub fn map_output(
        self: &mut Self,
        synth: &Synth,
        index: usize,
        bus: &AudioBus,
        flags: Methcla_BusMappingFlags,
    ) {
        self.add_msg(OscMessage {
            addr: "/synth/map/output".to_string(),
            args: Some(vec![
                OscType::Int(synth.node_id().0),
                OscType::Int(index as i32),
                OscType::Int(bus.bus_id()),
                OscType::Int(flags.0 as i32),
            ]),
        })
    }

    pub fn set(self: &mut Self, node: &Node, index: usize, value: f32) {
        self.add_msg(OscMessage {
            addr: "/node/set".to_string(),
            args: Some(vec![
                OscType::Int(node.node_id().0),
                OscType::Int(index as i32),
                OscType::Float(value),
            ]),
        })
    }

    pub fn free(self: &mut Self, node: &Node) {
        let id = node.node_id().0;
        self.add_msg(OscMessage {
            addr: "/node/free".to_string(),
            args: Some(vec![OscType::Int(id)]),
        });
        self.node_id_allocator.free(id)
    }

    pub fn when_done(self: &mut Self, synth: Synth, flags: Methcla_NodeDoneFlags) {
        self.add_msg(OscMessage {
            addr: "/synth/property/doneFlags/set".to_string(),
            args: Some(vec![
                OscType::Int(synth.node_id().0),
                OscType::Int(flags.0 as i32),
            ]),
        })
    }
}
