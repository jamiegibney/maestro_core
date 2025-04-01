use std::marker::PhantomData;

use super::*;

pub struct Arguments {
    pub osc_rx_port: u16,
    pub osc_tx_port: u16,

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

        unsafe {
            Ok(Self {
                osc_rx_port: rx_port.unwrap_unchecked(),
                osc_tx_port: tx_port.unwrap_unchecked(),

                _pd: PhantomData,
            })
        }
    }
}
