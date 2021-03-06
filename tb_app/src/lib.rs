use std::{env, io, path};
use tb_core::error::AnyError;
use tb_ecs::{Scheduler, SystemInfo, World};

pub struct Application {
    env_args: Vec<String>,
}

impl Application {
    pub fn new() -> Result<Application, AnyError> {
        Ok(Application {
            env_args: std::env::args().collect(),
        })
    }

    pub fn root_dir() -> Result<path::PathBuf, io::Error> {
        if let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") {
            return Ok(path::PathBuf::from(manifest_dir));
        }

        let mut exe = std::fs::canonicalize(env::current_exe()?)?;

        // Modify in-place to avoid an extra copy.
        if exe.pop() {
            return Ok(exe);
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to find an application root",
        ))
    }

    pub fn run(&mut self) {
        let mut scheduler = Scheduler::default();
        let mut world = World::default();

        loop {
            scheduler.schedule(&world);
        }
    }
}
