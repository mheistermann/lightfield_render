extern crate glium;

use std::io;
use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use glium::Program;
use glium::program::{ProgramCreationError, ProgramCreationInput};
use glium::backend::Facade;

use notify;
use notify::{RecommendedWatcher, Error, Watcher, Event, op};
use std::sync::mpsc::{channel, Receiver, TryRecvError};

#[derive(Debug)]
pub enum ProgramError {
    IOError(io::Error),
    GLError(ProgramCreationError),
}
impl From<io::Error> for ProgramError {
    fn from(err: io::Error) -> Self {
        ProgramError::IOError(err)
    }
}
impl From<ProgramCreationError> for ProgramError {
    fn from(err: ProgramCreationError) -> Self {
        ProgramError::GLError(err)
    }
}

struct ProgramInfo<'a> {
    // filenames
    vertex_shader_file: &'a str,
    fragment_shader_file: &'a str,
    geometry_shader_file: Option<&'a str>,
}

fn slurp(filename: &str) -> io::Result<String> {
    let mut f = try!(File::open(filename));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

impl<'a> ProgramInfo<'a> {
    pub fn create<F>(&self, facade: &F) -> Result<Program, ProgramError>  where F: Facade {
        let vertex_shader: String = try!(slurp(self.vertex_shader_file));
        let fragment_shader: String = try!(slurp(self.fragment_shader_file));
        let geom_content;
        let geometry_shader: Option<&str> = match self.geometry_shader_file {
            None => None,
            Some(filename) => {
                geom_content = try!(slurp(filename));
                Some(&geom_content)
            }
        };
        let prog = Program::from_source(facade, &vertex_shader, &fragment_shader, geometry_shader);
        return Ok(try!(prog));
    }
}

pub struct ReloadingProgram<'a, F: Facade + 'a> {
    current: Result<Program, ProgramError>,
    facade: &'a F,
    info: ProgramInfo<'a>,
    watcher: RecommendedWatcher,
    watcher_rx: Receiver<Event>,
}

impl<'a, F: Facade + 'a> ReloadingProgram<'a, F> {
    pub fn from_source(facade: &'a F,
                       vertex_shader_file: &'a str,
                       fragment_shader_file: &'a str,
                       geometry_shader_file: Option<&'a str>)
        -> ReloadingProgram<'a, F>
        where F: Facade
        {
            let (tx, rx) = channel();
            let w: Result<RecommendedWatcher, Error> = Watcher::new(tx);
            let mut watcher = w.unwrap();
            watcher.watch("shaders"); // XXX TODO FIXME
            watcher.watch(&vertex_shader_file).unwrap();
            watcher.watch(&fragment_shader_file).unwrap();
            geometry_shader_file.map(|x| watcher.watch(x).unwrap());
            let info = ProgramInfo {
                vertex_shader_file: vertex_shader_file,
                fragment_shader_file: fragment_shader_file,
                geometry_shader_file: geometry_shader_file,
            };
            return ReloadingProgram {
                facade: facade,
                watcher: watcher,
                watcher_rx: rx,
                current: info.create(facade),
                info: info,
            };
        }
    pub fn current(&mut self) -> &Result<Program, ProgramError> {
        let mut needs_recompile = false;
        loop {
            match self.watcher_rx.try_recv() {
                Err(TryRecvError::Empty) => { break},
                Err(TryRecvError::Disconnected) => { panic!("watcher disconnected")},
                Ok(Event{path, op}) => {
                    println!("{:?}", op);
                    needs_recompile = match op {
                        Ok(op::RENAME) => true,
                        Ok(op::CREATE) => true,
                        Ok(op::WRITE) => true,
                        _ => false,
                    }
                }
            }
        }
        if needs_recompile {
            println!("recompiling shaders");
            sleep(Duration::from_millis(100)); // XXX so hackish!
            println!("ok, waited long enough, let's do it!");
            let prog = self.info.create(self.facade);
            self.current = prog;
        }
        &self.current
    }
    pub fn wait_for_change(&self) -> () {
        for msg in self.watcher_rx.iter() {
            // TODO check message type
            return;
        }
    }
}
