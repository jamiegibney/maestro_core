use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use message::MIDIMessage;
use midir::{MidiIO, MidiOutput, MidiOutputPort};
use rand::seq::IndexedRandom;
use timer::TimerThread;

use super::*;

const MIDI_QUEUE_PREALLOC_SIZE: usize = 8192;

pub struct MIDISender {
    output: midir::MidiOutputConnection,
    port: midir::MidiOutputPort,
    bound_port_name: String,
    queue: Vec<u8>,
}

impl MIDISender {
    /// Returns a new `MIDISender` which binds to the first available MIDI port.
    ///
    /// # Errors
    ///
    /// Returns an error if a valid MIDI output could not be created, or if no
    /// MIDI port was found.
    pub fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let output = MidiOutput::new(name)?;

        if output.port_count() == 0 {
            return Err("no MIDI ports were found".into());
        }

        let mut ports = output.ports();

        let first_port = ports.remove(0);

        // println!(
        //     "binding to MIDI port {} with name \"{}\"",
        //     first_port.id(),
        //     output
        //         .port_name(&first_port)
        //         .unwrap_or_else(|_| String::from("UNKNOWN"))
        // );

        let bound_port_name = output
            .port_name(&first_port)
            .unwrap_or_else(|_| String::from("UNKNOWN"));

        let port_name = format!("{name}_port");

        let output = output.connect(&first_port, &port_name)?;

        Ok(Self {
            output,
            port: first_port,
            bound_port_name,
            queue: Vec::with_capacity(MIDI_QUEUE_PREALLOC_SIZE),
        })
    }

    /// Returns a new `MIDISender` which tries to bind to a port containing the
    /// provided substring. The search is case-insensitive.
    ///
    /// # Errors
    ///
    /// Returns an error if a valid MIDI output could not be created, if no
    /// MIDI port was found, or if no MIDI port contained the provided
    /// substring.
    pub fn new_with_port_containing(
        name: &str,
        port_substring: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let s = port_substring.to_lowercase();

        let output = MidiOutput::new(name)?;

        if output.port_count() == 0 {
            return Err("no MIDI ports were found".into());
        }

        let mut ports = output.ports();
        let mut port = None;

        for p in ports {
            if let Ok(name) = output.port_name(&p)
                && name.to_lowercase().contains(&s)
            {
                port = Some(p);
                break;
            }
        }

        if port.is_none() {
            return Err(format!(
                "no MIDI port contained the provided substring \"{s}\""
            )
            .into());
        }

        let port = unsafe { port.unwrap_unchecked() };

        // println!(
        //     "binding to MIDI port {} with name \"{}\"",
        //     port.id(),
        //     output
        //         .port_name(&port)
        //         .unwrap_or_else(|_| String::from("UNKNOWN"))
        // );

        let bound_port_name = output
            .port_name(&port)
            .unwrap_or_else(|_| String::from("UNKNOWN"));

        let port_name = format!("{name}_port");

        let output = output.connect(&port, &port_name)?;

        Ok(Self {
            output,
            port,
            bound_port_name,
            queue: Vec::with_capacity(MIDI_QUEUE_PREALLOC_SIZE),
        })
    }

    /// Enqueues the provided message to an internal queue, ready to be send via
    /// [`MIDISender::send_queue()`].
    pub fn enqueue(&mut self, message: &MIDIMessage) {
        if message.is_14_bit() {
            for byte in message.as_bytes_double() {
                self.queue.push(byte);
            }
        }
        else {
            for byte in message.as_bytes() {
                self.queue.push(byte);
            }
        }
    }

    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }

    pub fn queue(&self) -> &[u8] {
        &self.queue
    }

    /// Sends the internal queue of `MIDIMessage` bytes to the bound MIDI port.
    ///
    /// # Errors
    ///
    /// Returns an error if the MIDI message failed to send.
    pub fn send_queue(&mut self) -> Result<(), midir::SendError> {
        let result = self.output.send(&self.queue);

        self.queue.clear();
        Ok(())
    }

    /// Sends the provided `MIDIMessage` to the bound MIDI port.
    ///
    /// # Errors
    ///
    /// Returns an error if the MIDI message failed to send.
    pub fn send_direct(
        &mut self,
        message: &MIDIMessage,
    ) -> Result<(), midir::SendError> {
        if message.is_14_bit() {
            let msg = message.as_bytes_double();
            // let bytes = format!(
            //     "{:b} {:b} {:b} {:b} {:b} {:b}",
            //     msg[0], msg[1], msg[2], msg[3], msg[4], msg[5]
            // );
            // println!("sending {message} (bytes: {bytes})");

            self.output.send(&msg)
        }
        else {
            let msg = message.as_bytes();
            // let bytes = format!("{:b} {:b} {:b}", msg[0], msg[1], msg[2]);
            // println!("sending {message} (bytes: {bytes})");

            self.output.send(&msg)
        }
    }

    pub const fn get_port(&self) -> &MidiOutputPort {
        &self.port
    }

    pub fn close(self) {
        _ = self.output.close();
    }

    // hush now clippy for you are wrong in this case >:|
    #[allow(clippy::missing_const_for_fn)]
    pub fn bound_port_name(&self) -> &str {
        &self.bound_port_name
    }
}

// *** *** *** //

pub struct MIDISenderTimedThread {
    sender: Arc<Mutex<MIDISender>>,
    thread: TimerThread,
}

impl MIDISenderTimedThread {
    pub fn new(
        name: &str,
        substr: &str,
        receiver: CCReceiver<Vec<MIDIMessage>>,
    ) -> Result<Self, Box<dyn Error>> {
        let midi_sender = Arc::new(Mutex::new(
            MIDISender::new_with_port_containing(name, substr)?,
        ));

        let rx = Arc::new(Mutex::new(receiver));
        let tx = Arc::clone(&midi_sender);

        let thread = TimerThread::new(move || {
            if let Ok(mut receiver) = rx.lock()
                && let Ok(mut sender) = tx.lock()
            {
                while let Ok(buf) = receiver.try_recv()
                    && !buf.is_empty()
                {
                    for msg in &buf {
                        sender.enqueue(msg);
                    }

                    if let Err(e) = sender.send_queue() {
                        eprintln!("failed to send MIDI message: \"{e}\"");
                    }
                }
            }
        });

        Ok(Self { sender: midi_sender, thread })
    }

    pub fn start_send(&mut self) {
        self.thread.start_hz(MIDI_SEND_RATE);
    }

    pub fn stop_send(&mut self) {
        self.thread.stop(Some(1.0));
    }
}
