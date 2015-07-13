//! Interface to [HotSpot][1].
//!
//! [1]: http://lava.cs.virginia.edu/HotSpot

extern crate libc;

use std::ffi::CString;
use std::fs;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;

mod ffi;

macro_rules! raise(
    ($kind:ident, $message:expr) => (
        return Err(Error::new(ErrorKind::$kind, $message))
    );
);

macro_rules! str_to_c_str(
    ($str:expr) => (
        match CString::new($str) {
            Ok(result) => result,
            Err(_) => raise!(Other, "failed to process the arguments"),
        }
    );
);

macro_rules! path_to_c_str(
    ($path:expr) => (
        match $path.to_str() {
            Some(path) => str_to_c_str!(path),
            None => raise!(Other, "failed to process the arguments"),
        }
    );
);

/// A thermal circuit.
pub struct Circuit {
    /// The number of processing elements.
    pub cores: usize,
    /// The number of thermal nodes.
    pub nodes: usize,
    /// An `nodes`-element vector of thermal capacitance.
    pub capacitance: Vec<f64>,
    /// An `nodes`-by-`nodes` matrix of thermal conductance.
    pub conductance: Vec<f64>,
}

impl Circuit {
    /// Construct a thermal circuit.
    ///
    /// The only supported model is the block model.
    pub fn new<F: AsRef<Path>, C: AsRef<Path>>(floorplan: F, config: C) -> Result<Circuit> {
        use std::ptr::copy_nonoverlapping as copy;

        let (floorplan, config) = (floorplan.as_ref(), config.as_ref());
        if fs::metadata(floorplan).is_err() {
            raise!(NotFound, "the floorplan file does not exist");
        }
        if fs::metadata(config).is_err() {
            raise!(NotFound, "the configuration file does not exist");
        }

        unsafe {
            let floorplan = path_to_c_str!(floorplan);
            let config = path_to_c_str!(config);

            let circuit = ffi::new_Circuit(floorplan.as_ptr(), config.as_ptr());
            if circuit.is_null() {
                raise!(Other, "failed to construct a thermal circuit");
            }

            let circuit = &*circuit;

            let cores = circuit.cores as usize;
            let nodes = circuit.nodes as usize;
            let mut capacitance = vec![0.0; nodes];
            let mut conductance = vec![0.0; nodes * nodes];

            copy(circuit.capacitance as *const _, capacitance.as_mut_ptr(), nodes);
            copy(circuit.conductance as *const _, conductance.as_mut_ptr(), nodes * nodes);

            ffi::drop_Circuit(circuit as *const _ as *mut _);

            Ok(Circuit {
                cores: cores,
                nodes: nodes,
                capacitance: capacitance,
                conductance: conductance,
            })
        }
    }
}
