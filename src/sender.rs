use anyhow::Result as AnyResult;
use anyhow::anyhow;
use mpsc::{Receiver, Sender};
use serialport;
use std::fmt;
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};
use std::num::ParseIntError;
use std::ops::DerefMut;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;
const DEFAULT_TIMEOUT: u64 = 30000;

#[derive(Debug)]
pub enum PlotterConnectionError {
    #[allow(dead_code)]
    IOError(i32),
    DeviceError(String),
    ParseError(String),
    UnknownError,
}

impl From<url::ParseError> for PlotterConnectionError {
    fn from(error: url::ParseError) -> Self {
        PlotterConnectionError::DeviceError(error.to_string())
    }
}

impl From<ParseIntError> for PlotterConnectionError {
    fn from(error: ParseIntError) -> Self {
        PlotterConnectionError::ParseError(error.to_string())
    }
}

impl From<serialport::Error> for PlotterConnectionError {
    fn from(error: serialport::Error) -> Self {
        PlotterConnectionError::DeviceError(error.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlotterCommand {
    Connect(String), // A URL to connect to (/dev/ttyACM0, telnet://foo:1234, etc)
    Disconnect,
    Program(Box<Vec<String>>),
    Run,
    Stop,
    Reset,
    Command(String),
    Shutdown,
    Ping,
}

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub enum PlotterState {
    #[default]
    Disconnected,
    Connecting(String),
    Ready,
    Running(u32, u32, u32), // Line of lines
    Paused(u32, u32, u32),  // Paused at a point in time.
    Busy,                   // Running a command.
    Failed(String),
    Terminating,
    Dead,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlotterResponse {
    Ok(PlotterCommand, String),
    Loaded(String),
    Err(PlotterCommand, String),
    State(PlotterState), // progress and msg
}

pub struct PlotterConnection {
    // send: Sender<PlotterResponse>,
    // recv: Receiver<PlotterCommand>,
    transport: Option<TransportTypes>,
    program: Option<Box<Vec<String>>>,
    // position: Option<usize>,
    state: PlotterState,
    recv: Receiver<PlotterCommand>,
    send: Sender<PlotterResponse>,
    ticks: usize,
    oks: usize,
}

impl fmt::Debug for PlotterConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlotterConnection: transport {:?}", &self.transport)
    }
}

impl PlotterConnection {
    pub fn spawn() -> AnyResult<(Sender<PlotterCommand>, Receiver<PlotterResponse>)> {
        let (cmdsend, cmdrecv) = mpsc::channel();
        let (respsend, resprecv) = mpsc::channel();
        std::thread::spawn(move || {
            respsend
                .send(PlotterResponse::Ok(
                    PlotterCommand::Reset,
                    "Started".to_string(),
                ))
                .expect("Cannot send OK response to parent thread");
            let mut me = PlotterConnection {
                transport: None,
                program: None,
                // position: None,
                state: PlotterState::default(),
                send: respsend,
                recv: cmdrecv,
                ticks: 0,
                oks: 0,
            };
            me.run();
        });
        Ok((cmdsend, resprecv))
    }

    fn wait_ok(&mut self) -> AnyResult<()> {
        let tx: &mut TransportTypes = self.transport.as_mut().unwrap(); // I literally just set it.
        let mut banner = String::with_capacity(80);
        loop {
            banner.clear();
            match tx.read_line(&mut banner) {
                Ok(count) => {
                    if count == 0usize {
                        println!("EOF");
                        return Err(anyhow!("EOF"));
                    } else {
                        if banner.starts_with("ok") {
                            return Ok(());
                        } else if banner.starts_with("!!") {
                            return Err(anyhow!("Plotter error: {}", &banner));
                        }
                    };
                }
                Err(_) => {
                    // Ruh Roh
                    return Err(anyhow!("EOF"));
                }
            }
        }
    }

    pub fn on_recv(&mut self, message: &PlotterCommand) {
        // println!("ON_RECV: {:?}", &message);
        match message {
            PlotterCommand::Ping => {
                println!("PING!");
                self.send
                    .send(PlotterResponse::Ok(message.clone(), "PONG!".to_string()))
                    .expect("Cannot send OK result");
            }
            PlotterCommand::Connect(conn_detail) => {
                println!("!connect");
                self.set_state(PlotterState::Connecting(conn_detail.clone()))
                    .expect("Cannot set to connecting?");
                match TransportTypes::from_uri(conn_detail.as_str()) {
                    Ok(transport) => {
                        self.transport = Some(transport);
                        self.send
                            .send(PlotterResponse::Ok(
                                message.clone(),
                                "Connected".to_string(),
                            ))
                            .expect("Failed to send response?");
                        if let Ok(_) = self.wait_ok() {
                            self.set_state(PlotterState::Ready)
                                .expect("Failed to set state?");
                        } else {
                            self.set_state(PlotterState::Disconnected)
                                .expect("Failed to set state?");
                        }
                    }
                    Err(err) => {
                        println!("Failed to connect: {:?}", &err);
                        self.transport = None;
                        self.send
                            .send(PlotterResponse::Err(
                                message.clone(),
                                format!("Failed to connect: {:?}", err),
                            ))
                            .expect("Failed to send response?");
                        self.set_state(PlotterState::Disconnected)
                            .expect("Failed to set state?");
                    }
                }
                println!("Done connect wait.");
            }
            PlotterCommand::Disconnect => match self.state {
                PlotterState::Ready => {
                    self.transport = None;
                    self.set_state(PlotterState::Disconnected)
                        .expect("Cannot send disconnected state.");
                    self.send
                        .send(PlotterResponse::Ok(
                            message.clone(),
                            "Disconnected.".to_string(),
                        ))
                        .expect("Cannot send OK result");
                }
                _ => {
                    self.send
                        .send(PlotterResponse::Err(
                            message.clone(),
                            format!("Invalid state for disconnect: {:?}", self.state.clone()),
                        ))
                        .expect("Cannot send error result");
                }
            },
            PlotterCommand::Program(program) => {
                self.program = Some(program.clone());
                self.send
                    .send(PlotterResponse::Loaded(format!(
                        "Loaded {} lines.",
                        program.len()
                    )))
                    .expect("Cannot send OK response to parent thread");
            }
            PlotterCommand::Run => match &self.state {
                PlotterState::Running(_line, _lines, _oks) => {}
                PlotterState::Paused(line, lines, _oks) => {
                    self.set_state(PlotterState::Running(*line, *lines, self.oks as u32))
                        .expect("Couldn't set resume running state after pause.");
                    self.send
                        .send(PlotterResponse::Ok(
                            message.clone(),
                            "Program resumed.".to_string(),
                        ))
                        .expect("Cannot send OK response to parent thread");
                }
                PlotterState::Ready => {
                    match &self.program {
                        Some(program) => {
                            self.oks = 0;
                            self.set_state(PlotterState::Running(
                                0,
                                program.len() as u32,
                                self.oks as u32,
                            ))
                            .expect("Couldn't set state to running!");
                            self.send
                                .send(PlotterResponse::Ok(
                                    message.clone(),
                                    "Program started.".to_string(),
                                ))
                                .expect("Cannot send OK response to parent thread");
                        }
                        None => todo!(),
                    };
                }
                _ => {}
            },
            PlotterCommand::Stop => match &self.state {
                PlotterState::Running(line, lines, oks) => {
                    self.set_state(PlotterState::Paused(*line, *lines, *oks))
                        .expect("Cannot set paused state in sender thread.");
                    self.send
                        .send(PlotterResponse::Ok(
                            message.clone(),
                            "Disconnected.".to_string(),
                        ))
                        .expect("Cannot send OK response to parent thread");
                }
                _ => {
                    self.send
                        .send(PlotterResponse::Err(
                            message.clone(),
                            format!("Invalid state {:?} for stop command.", &self.state)
                                .to_string(),
                        ))
                        .expect("Cannot send OK response to parent thread");
                }
            },
            PlotterCommand::Reset => {
                println!("Got reset.");
                self.transport = None;
                self.set_state(PlotterState::Disconnected)
                    .expect("Cannot send disconnected state to parent thread");
                self.send
                    .send(PlotterResponse::Ok(
                        message.clone(),
                        "Disconnected.".to_string(),
                    ))
                    .expect("Cannot send OK response to parent thread");
            }
            PlotterCommand::Command(cmd) => {
                self.set_state(PlotterState::Busy)
                    .expect("Failed to set busy state.");
                match &mut self.transport {
                    Some(transport) => {
                        // println!("Sending command: '{}'", &cmd);
                        transport.write_line(&cmd).expect("Failed to send line.");
                        transport.flush().expect("Failed to flush line.");
                        if let Ok(_) = self.wait_ok() {
                            self.set_state(PlotterState::Ready)
                                .expect("Failed to set state?");
                        } else {
                            self.set_state(PlotterState::Disconnected)
                                .expect("Failed to set state?");
                        }
                    }
                    None => {
                        self.set_state(PlotterState::Failed(
                            "Invalid state! No transport in 'Ready' state.".to_string(),
                        ))
                        .expect("Couldn't send failure message.");
                        self.send
                            .send(PlotterResponse::Err(message.clone(), "Failed".to_string()))
                            .expect("response queue closed");
                    }
                };
            }
            PlotterCommand::Shutdown => {
                self.set_state(PlotterState::Terminating)
                    .expect("Failed to send response");

                self.send
                    .send(PlotterResponse::Ok(
                        message.clone(),
                        "Terminating".to_string(),
                    ))
                    .expect("Failed to send response");
            }
        }
    }

    pub fn set_state(
        &mut self,
        state: PlotterState,
    ) -> Result<(), mpsc::SendError<PlotterResponse>> {
        self.state = state;
        self.send.send(PlotterResponse::State(self.state.clone()))
    }

    pub fn run(&mut self) {
        //}, recv: Receiver<PlotterCommand>, send: Sender<PlotterResponse>) {
        loop {
            self.ticks += 1;
            match self.recv.try_recv() {
                Ok(message) => self.on_recv(&message),
                Err(err) => match err {
                    TryRecvError::Empty => (),
                    TryRecvError::Disconnected => {
                        println!("Socket disconnected?!");
                        self.set_state(PlotterState::Dead)
                            .expect("Cannot send DEAD state to parent thread");
                    }
                },
            }
            match self.state {
                PlotterState::Disconnected => {
                    std::thread::sleep(std::time::Duration::from_millis(100))
                }
                PlotterState::Connecting(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(100))
                }
                PlotterState::Ready => std::thread::sleep(std::time::Duration::from_millis(100)),
                PlotterState::Paused(_, _, _) => {
                    std::thread::sleep(std::time::Duration::from_millis(100))
                }
                PlotterState::Running(current_line, total_lines, _oks) => {
                    if self.oks < 5 {
                        match &mut self.transport {
                            Some(transport) => {
                                // println!("Transport: {:?}", &transport);
                                if let Some(program) = &self.program {
                                    if let Some(line) = program.get(current_line as usize) {
                                        // println!("-->> {}", &line);
                                        match transport.write_line(line) {
                                            Ok(_) => {
                                                transport
                                                    .flush()
                                                    .expect("Cannot flush commands to plotter.");
                                                self.set_state(PlotterState::Running(
                                                    current_line + 1u32,
                                                    total_lines.clone(),
                                                    self.oks as u32,
                                                ))
                                                .expect("Failed to update state");
                                                self.oks += 1;
                                            }
                                            Err(err) => {
                                                self.set_state(PlotterState::Failed(format!(
                                                    "Failed to plot: {:?}",
                                                    err
                                                )))
                                                .expect("Cloudn't set error/fail state");
                                            }
                                        }
                                    } else {
                                        println!("No lines left.");
                                    }
                                } else {
                                    println!("Program done?!");
                                }
                            }
                            None => {
                                // Failed! Somedthing broked.
                                self.transport = None;
                                self.set_state(PlotterState::Disconnected)
                                    .expect("Can't send state update.");
                            }
                        }

                        // Next, wait for outstanding OKs
                        if self.oks > 0 {
                            // println!("Still waiting for {} OKs.", self.oks);
                            match &mut self.transport {
                                Some(_transport) => match self.wait_ok() {
                                    Ok(_) => {
                                        self.oks -= 1;
                                    }

                                    Err(_err) => {
                                        self.set_state(PlotterState::Disconnected)
                                            .expect("Cannot set state after failure.");
                                    }
                                },
                                None => self
                                    .set_state(PlotterState::Disconnected)
                                    .expect("No transport!!! Disconnecting."),
                            }
                        }
                        if self.oks == 0 && current_line == total_lines {
                            self.set_state(PlotterState::Ready)
                                .expect("Can't go back to ready state.");
                        }
                    }
                }
                PlotterState::Failed(_) => {
                    eprintln!("Plotter state->failed received.");
                    self.set_state(PlotterState::Disconnected)
                        .expect("Can't die?!");
                }
                PlotterState::Terminating => {
                    self.set_state(PlotterState::Dead).expect("Can't die?!");
                    break;
                }
                PlotterState::Dead => {
                    println!("Died!");
                    break; // I died or lost my receiver, so bailing out.
                }
                PlotterState::Busy => std::thread::sleep(std::time::Duration::from_millis(100)),
            }
        }
        println!("Exited plot connection runner.");
    }
}

pub trait PlotterTransport {
    fn write_line(&mut self, buf: &str) -> std::io::Result<()>;
    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize>;
    fn flush(&mut self) -> std::io::Result<()>;
}

pub enum TransportTypes {
    SerialReadWrite(Box<dyn BufRead>, Box<dyn Write>),
}

impl TransportTypes {
    /// Given a URI in the form of serial:///dev/ttySomethingOrOther@115200,
    /// open up a serial connection on the /dev/ttySomethingOrOther at 115200 bps.
    pub fn from_uri(uri: &str) -> Result<TransportTypes, PlotterConnectionError> {
        let url = url::Url::parse(uri)?;
        if url.scheme() == "serial" {
            let mut parts: Vec<&str> = url.path().split("@").collect();
            if parts.len() == 1 {
                parts.push("115200"); // default to 115200 baud
            }
            if parts.len() == 2 {
                let path = parts[0].to_string();
                let bps = parts[1].to_string().parse::<u32>()?;
                let sp = serialport::new(path, bps)
                    .timeout(Duration::from_millis(DEFAULT_TIMEOUT))
                    .open()?;
                let reader = BufReader::new(sp.try_clone()?);
                let writer = BufWriter::new(sp);
                Ok(TransportTypes::SerialReadWrite(
                    Box::new(reader),
                    Box::new(writer),
                ))
            } else {
                Err(PlotterConnectionError::UnknownError)
            }
        } else {
            Err(PlotterConnectionError::UnknownError)
        }
    }
}

impl fmt::Debug for TransportTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PlotterTransport: {}",
            match self {
                TransportTypes::SerialReadWrite(_, _) => "Serial",
            }
        )
    }
}

impl PlotterTransport for TransportTypes {
    fn write_line(&mut self, buf: &str) -> std::io::Result<()> {
        // println!("SENDING: '{}'", buf);
        match self {
            TransportTypes::SerialReadWrite(_, bwrite) => bwrite
                .deref_mut()
                .write_all((buf.to_owned() + "\n").as_bytes()),
        }
    }

    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
        match self {
            TransportTypes::SerialReadWrite(bread, _) => bread.deref_mut().read_line(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            TransportTypes::SerialReadWrite(_, bwrite) => bwrite.deref_mut().flush(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    /*
    #[tests]
    fn test_from_url(){
        let mut pc=PlotterConnection::from_uri("serial:///dev/foobar@38400").expect("This sucks");
        &pc.write_line("FAILED");
        let flushed = &pc.flush();
        println!("Flush result: {:?}", flushed);
        match flushed {
            Ok(_) => assert!(false), // We failed the tests
            Err(err) => println!("The failure: {}", err.to_string())
        }

    }
    */

    #[test]
    fn test_happy_path() {
        let (cmdsend, resprecv) = PlotterConnection::spawn().unwrap();

        // println!("Spawned");
        cmdsend.send(PlotterCommand::Connect(
            "serial:///dev/ttyACM0@1000000".to_string(),
        ));
        std::thread::sleep_ms(1000);
        let program = Box::new(vec![
            "G28 X Y".to_string(),
            "M280 S5".to_string(),
            "G0 X10 Y10".to_string(),
            "M280 S12".to_string(),
            "G01 F1200".to_string(),
            "X50".to_string(),
            "Y50".to_string(),
            "X10".to_string(),
            "Y10".to_string(),
            "M400".to_string(),
            "G4 P1000".to_string(),
            "M280 S5".to_string(),
            "G0 X0 Y0".to_string(),
            "M400".to_string(),
        ]);
        cmdsend.send(PlotterCommand::Program(program));
        cmdsend.send(PlotterCommand::Run);

        loop {
            if let Ok(resp) = resprecv.recv() {
                // println!("Response:");
                // println!("\t{:?}", &resp);
                match (resp) {
                    PlotterResponse::Ok(_, _) => (),
                    PlotterResponse::Loaded(_) => println!("Program loaded."),
                    PlotterResponse::Err(_, _) => break,
                    PlotterResponse::State(_) => (),
                }
            }
        }
        cmdsend.send(PlotterCommand::Shutdown);
    }

    #[test]
    fn test_early_termination() {
        let (cmdsend, resprecv) = PlotterConnection::spawn().unwrap();
        println!("Spawned");
        cmdsend.send(PlotterCommand::Connect(
            "serial:///dev/ttyACM0@1000000".to_string(),
        ));
        std::thread::sleep_ms(1000);
        cmdsend.send(PlotterCommand::Command("G28 X Y ;".to_string()));

        cmdsend.send(PlotterCommand::Shutdown);
        for response in resprecv {
            println!("Response:");
            println!("\t{:?}", &response);
        }
    }

    #[test]
    fn test_trial() {
        // let mut sp = serialport::new("/dev/tty.usbmodem14122301", 115200)
        //     .timeout(Duration::from_millis(5000))
        //     .open()
        //     .expect("Failed to open");
        // let mut reader = BufReader::new(sp.try_clone().expect("Failed to clone port"));
        // let mut writer = BufWriter::new(sp);
        // println!("Connected.");
        // let mut pc = PlotterConnection::SerialReadWrite(Box::new(reader), Box::new(writer));
        /*
        let mut pc =
            TransportTypes::from_uri("serial:///dev/tty.usbmodem14122301@115200").expect("Failed!");

        //&pc.write_line("M999");
        &pc.write_line("M280 S8");
        &pc.write_line("G28 X Y");
        &pc.write_line("G90");
        &pc.write_line("G92 X0 Y0");
        &pc.write_line("G4 P150");
        &pc.write_line("M281");

        // &pc.write_line("G0 F100");
        // &pc.write_line("G1 F100");
        // &pc.write_line("M281");
        // &pc.write_line("G92 X0 Y0");
        // &pc.write_line("G90");
        // &pc.write_line("G92 X0 Y0");

        &pc.flush();
        let mut buf = String::new();
        for i in 0..6 {
            &pc.read_line(&mut buf).expect("Failed to read...");
            println!("{}> '{}'", i, buf.trim());
            buf.clear();
        }
        */
        // loop{
        //     &pc.read_line(&mut buf).expect("Overage");
        //     println!("Extra line: '{}'", buf.trim());
        //     buf.clear();
        // }
        /*        println!("Read: {} bytes",pc.read_line(&mut buf).expect("Failed to read..."));
                println!("1: {}", buf.trim());
                assert_eq!(buf.trim(), "ok");
                println!("Read {} bytes", pc.read_line(&mut buf).expect("Failed to read..."));
                println!("2: {}", buf.trim());
                assert_eq!(buf.trim(), "ok");
        */
    }
}
