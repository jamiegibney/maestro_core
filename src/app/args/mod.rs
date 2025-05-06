use std::marker::PhantomData;

use super::*;

#[allow(clippy::struct_excessive_bools)]
pub struct Arguments {
    pub osc_rx_port: u16,
    pub osc_tx_port: u16,
    pub auto_start_send: bool,
    pub show_state_data: bool,
    pub auto_change_mode: bool,
    pub print: bool,
    pub debug: bool,

    _pd: PhantomData<()>,
}

impl Arguments {
    pub fn from_env() -> Result<Self, String> {
        let mut args = std::env::args();

        _ = args.next();

        let first_arg = args.next();

        if first_arg.is_none() {
            return Err(String::from("no arguments were received"));
        }

        let second_arg = args.next();

        if second_arg.is_none() {
            return Err(String::from("missing second argument for TX port"));
        }

        let rx_port = unsafe { first_arg.unwrap_unchecked().parse::<u16>() };
        if let Err(e) = rx_port {
            return Err(e.to_string());
        }

        let tx_port = unsafe { second_arg.unwrap_unchecked().parse::<u16>() };
        if let Err(e) = tx_port {
            return Err(e.to_string());
        }

        let mut auto_start_send = false;
        let mut show_state_data = true;
        let mut auto_change_mode = true;
        let mut print = true;
        let mut debug = false;

        for mut arg in args {
            arg = arg.to_lowercase();

            if arg.contains("--auto-start") {
                auto_start_send = true;
            }

            if arg.contains("--no-ui") {
                show_state_data = false;
            }

            if arg.contains("--static-mode") {
                auto_change_mode = false;
            }

            if arg.contains("--quiet") {
                print = false;
            }

            if arg.contains("--debug") {
                debug = true;
            }
        }

        unsafe {
            Ok(Self {
                osc_rx_port: rx_port.unwrap_unchecked(),
                osc_tx_port: tx_port.unwrap_unchecked(),
                auto_start_send,
                show_state_data,
                auto_change_mode,
                print,
                debug,

                _pd: PhantomData,
            })
        }
    }
}
