use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
};

use super::*;

use args::Arguments;
use eme_request::{EMERequest, ToJson};
use nannou_osc::{self as osc, Connected};
use timer::TimerThread;

pub mod eme_request;

pub const OSC_IP_ADDRESS: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const MAX_OSC_SEND_ATTEMPTS: usize = 16;

fn addr_string(port: u16) -> String {
    format!("127.0.0.1:{port}")
}

// *** *** *** //

pub struct OSCReceiver {
    receiver: osc::Receiver,
}

impl OSCReceiver {
    pub fn with_port(port: u16) -> std::io::Result<Self> {
        Ok(Self { receiver: osc::receiver(port)? })
    }

    pub fn try_recv(&mut self) -> Option<osc::Packet> {
        let mut packet = None;

        while let Ok(opt) = self.receiver.try_recv()
            && let Some((p, _)) = opt
        {
            packet = Some(p);
        }

        packet
    }
}

// *** *** *** //

pub struct EMERequestOSCSender {
    sender: Arc<Mutex<osc::Sender<Connected>>>,
    osc_sender_timer: TimerThread,
}

impl EMERequestOSCSender {
    pub fn new(
        port: u16,
        eme_request_channel: CCReceiver<EMERequest>,
    ) -> std::io::Result<Self> {
        // let tx_addr = SocketAddr::new(OSC_IP_ADDRESS, port);
        let sender = osc::sender()?.connect(addr_string(port))?;
        let sender = Arc::new(Mutex::new(sender));

        // timer thread state
        let osc_sender = Arc::clone(&sender);
        let request_rx = Arc::new(Mutex::new(eme_request_channel));

        let osc_sender_timer = TimerThread::new(move || {
            let osc_addr = EME_OSC_REQUEST_CHANNEL.to_string();

            if let Ok(osc) = osc_sender.lock()
                && let Ok(receiver) = request_rx.lock()
            {
                while let Ok(eme_request) = receiver.try_recv() {
                    let request_str = eme_request.as_json().to_string();
                    let args = Vec::from([osc::Type::String(request_str)]);

                    let mut send_result =
                        osc.send((osc_addr.clone(), args.clone()));

                    let mut attempts = 1;

                    while let Err(e) = &send_result
                        && attempts < MAX_OSC_SEND_ATTEMPTS
                    {
                        attempts += 1;
                        eprintln!("failed to send OSC message (attempt #{attempts}): {e}");

                        send_result =
                            osc.send((osc_addr.clone(), args.clone()));
                    }
                }
            }
        });

        Ok(Self { sender, osc_sender_timer })
    }

    pub fn start_send(&mut self) {
        self.osc_sender_timer.start_hz(OSC_SEND_RATE);
    }

    pub fn stop_send(&mut self) {
        self.osc_sender_timer.stop(Some(1.0));
    }
}

// *** *** *** //

pub fn create_osc_sender_and_receiver(
    args: &Arguments,
    eme_request_channel: CCReceiver<EMERequest>,
) -> std::io::Result<(EMERequestOSCSender, OSCReceiver)> {
    Ok((
        EMERequestOSCSender::new(args.osc_tx_port, eme_request_channel)?,
        OSCReceiver::with_port(args.osc_rx_port)?,
    ))
}
