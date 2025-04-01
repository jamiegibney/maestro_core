use args::Arguments;
use hand_parser::HandParser;
use hand_types::{
    COMPair, RawHand, RawHandPair, RawHandPairCOM, ValidRawHandPair,
};
use osc::OSCReceiver;

use super::*;

mod hand_parser;
pub mod hand_types;

pub const NUM_HAND_VERTICES: usize = 21;
pub const HAND_DETECTION_TIMEOUT: f64 = 1.0;
pub const HAND_DAMPING_TIME: f64 = 0.020;
pub const MAX_HAND_SPEED: f64 = 250.0;

pub const MAX_HAND_VELOCITY: f64 = 100.0;

pub const WRIST_VERTEX_INDEX: usize = 0;
pub const THUMB_TIP_VERTEX_INDEX: usize = 4;
pub const INDEX_TIP_VERTEX_INDEX: usize = 8;
pub const MIDDLE_TIP_VERTEX_INDEX: usize = 12;
pub const RING_TIP_VERTEX_INDEX: usize = 16;
pub const PINKY_TIP_VERTEX_INDEX: usize = 20;

pub const fn outer_hand_vertex_indices() -> [usize; 6] {
    [
        WRIST_VERTEX_INDEX, THUMB_TIP_VERTEX_INDEX, INDEX_TIP_VERTEX_INDEX,
        MIDDLE_TIP_VERTEX_INDEX, RING_TIP_VERTEX_INDEX, PINKY_TIP_VERTEX_INDEX,
    ]
}

fn is_outer_vertex(idx: usize) -> bool {
    outer_hand_vertex_indices().contains(&idx)
}

const fn color(r: f32, g: f32, b: f32, a: f32) -> Rgba {
    Rgba {
        color: rgb::Rgb {
            red: r,
            green: g,
            blue: b,
            standard: std::marker::PhantomData,
        },
        alpha: a,
    }
}

pub const UNKNOWN_HAND_COLOR: Rgba = color(1.0, 1.0, 1.0, 0.5);
pub const OPEN_HAND_COLOR: Rgba = color(0.0, 1.0, 0.0, 1.0);
pub const CLOSED_HAND_COLOR: Rgba = color(1.0, 0.0, 0.0, 1.0);
pub const THUMB_UP_DOWN_HAND_COLOR: Rgba = color(1.0, 1.0, 0.0, 1.0);

pub const DEFAULT_COM_COLOR: Rgba = color(0.2, 0.5, 1.0, 0.15);

#[derive(Debug, Default)]
struct InvalidHandTimeout {
    first: Option<f64>,
    first_to: bool,
    second: Option<f64>,
    second_to: bool,
}

pub struct HandManager {
    parser: HandParser,
    osc_receiver: OSCReceiver,

    damped_hands: RawHandPairCOM,
    hand_velocity: ValidRawHandPair,

    latest_target: ValidRawHandPair,
    invalid_timeout: InvalidHandTimeout,

    can_update: bool,
}

impl HandManager {
    pub fn new(osc_receiver: OSCReceiver) -> Self {
        Self {
            parser: HandParser::new(),
            osc_receiver,

            damped_hands: RawHandPairCOM::default(),
            hand_velocity: ValidRawHandPair::default(),

            latest_target: ValidRawHandPair::default(),
            invalid_timeout: InvalidHandTimeout {
                first: None,
                first_to: false,
                second: None,
                second_to: false,
            },

            can_update: false,
        }
    }

    pub const fn damped_hands(&self) -> &RawHandPairCOM {
        &self.damped_hands
    }

    pub fn start_update(&mut self) {
        self.can_update = true;
    }

    pub fn stop_update(&mut self) {
        self.can_update = false;
    }

    fn update_from(&mut self, target_hands: &RawHandPair, delta_time: f64) {
        if let Some(first) = &target_hands.first {
            self.latest_target.first = *first;
            self.invalid_timeout.first = None;
            self.invalid_timeout.first_to = false;
        }
        else if self.invalid_timeout.first.is_none() {
            self.invalid_timeout.first = Some(0.0);
        }

        if let Some(second) = &target_hands.second {
            self.latest_target.second = *second;
            self.invalid_timeout.second = None;
            self.invalid_timeout.second_to = false;
        }
        else if self.invalid_timeout.second.is_none() {
            self.invalid_timeout.second = Some(0.0);
        }

        self.update_invalid(delta_time);

        self.update_first(delta_time);
        self.update_second(delta_time);
    }

    fn update_invalid(&mut self, delta_time: f64) {
        if let Some(l) = &mut self.invalid_timeout.first {
            *l += delta_time;

            if !self.invalid_timeout.first_to && *l > HAND_DETECTION_TIMEOUT {
                self.reset_first();
            }
        }

        if let Some(r) = &mut self.invalid_timeout.second {
            *r += delta_time;

            if !self.invalid_timeout.second_to && *r > HAND_DETECTION_TIMEOUT {
                self.reset_second();
            }
        }
    }

    fn reset_first(&mut self) {
        self.invalid_timeout.first_to = true;
        self.damped_hands.pair.first = None;
        self.damped_hands.com.first = None;
        self.hand_velocity.first = RawHand::default();
    }

    fn reset_second(&mut self) {
        self.invalid_timeout.second_to = true;
        self.damped_hands.pair.second = None;
        self.damped_hands.com.second = None;
        self.hand_velocity.second = RawHand::default();
    }

    fn update_first(&mut self, delta_time: f64) {
        if self.damped_hands.pair.first.is_none() {
            if self.invalid_timeout.first.is_some() {
                return;
            }

            self.damped_hands.pair.first = Some(self.latest_target.first);
        }

        if let Some(curr) = self.damped_hands.pair.first.as_mut() {
            for (i, p) in curr.points.iter_mut().enumerate() {
                *p = Self::smooth_damp_vec3(
                    *p, self.latest_target.first.points[i],
                    &mut self.hand_velocity.first.points[i], HAND_DAMPING_TIME,
                    delta_time, MAX_HAND_SPEED,
                );
            }

            curr.gesture = self.latest_target.first.gesture;
        }
    }

    fn update_second(&mut self, delta_time: f64) {
        if self.damped_hands.pair.second.is_none() {
            if self.invalid_timeout.second.is_some() {
                return;
            }

            self.damped_hands.pair.second = Some(self.latest_target.second);
        }

        if let Some(curr) = self.damped_hands.pair.second.as_mut() {
            for (i, p) in curr.points.iter_mut().enumerate() {
                *p = Self::smooth_damp_vec3(
                    *p, self.latest_target.second.points[i],
                    &mut self.hand_velocity.second.points[i],
                    HAND_DAMPING_TIME, delta_time, MAX_HAND_SPEED,
                );
            }

            curr.gesture = self.latest_target.second.gesture;
        }
    }

    fn smooth_damp_vec3(
        curr: DVec3,
        target: DVec3,
        vel: &mut DVec3,
        time: f64,
        delta_time: f64,
        max_speed: f64,
    ) -> DVec3 {
        let mut out = DVec3::default();

        out.x = smooth_damp(
            curr.x, target.x, &mut vel.x, time, delta_time, max_speed,
        );
        out.y = smooth_damp(
            curr.y, target.y, &mut vel.y, time, delta_time, max_speed,
        );
        out.z = smooth_damp(
            curr.z, target.z, &mut vel.z, time, delta_time, max_speed,
        );

        out
    }
}

impl Updatable for HandManager {
    fn update(&mut self, update: &Update) {
        if !self.can_update {
            return;
        }
        
        let latest_packet = self.osc_receiver.try_recv();

        if latest_packet.is_none() {
            self.update_from(
                &RawHandPair::default(),
                update.since_last.as_secs_f64(),
            );
            self.damped_hands.update(update);

            return;
        }

        let packet = unsafe { latest_packet.unwrap_unchecked() };

        let hands = self.parser.parse_hands(packet);

        if let Err(e) = &hands {
            println!("FAILED to parse hands: {e}");
            return;
        }

        let hands = unsafe { hands.unwrap_unchecked() };

        self.update_from(&hands, update.since_last.as_secs_f64());
        self.damped_hands.update(update);
    }
}
