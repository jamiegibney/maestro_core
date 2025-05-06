use std::{mem::transmute, ops::Rem};

use nannou::color::{Alpha, IntoColor};
use rand::seq::IndexedRandom;

use crate::util;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

impl Finger {
    pub fn index(self) -> usize {
        match self {
            Self::Thumb => THUMB_TIP_VERTEX_INDEX,
            Self::Index => INDEX_TIP_VERTEX_INDEX,
            Self::Middle => MIDDLE_TIP_VERTEX_INDEX,
            Self::Ring => RING_TIP_VERTEX_INDEX,
            Self::Pinky => PINKY_TIP_VERTEX_INDEX,
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandGesture {
    #[default]
    Unknown,
    Open,
    Closed,
    ThumbUp,
    ThumbDown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalPairOrder {
    FirstLeftSecondRight,
    FirstRightSecondLeft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalPairOrder {
    FirstTopSecondBottom,
    FirstBottomSecondTop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairOrder {
    FirstAboveAndLeft,
    FirstAboveAndRight,
    FirstBelowAndLeft,
    FirstBelowAndRight,
}

impl PairOrder {
    pub const fn from(
        hor: HorizontalPairOrder,
        ver: VerticalPairOrder,
    ) -> Self {
        match hor {
            HorizontalPairOrder::FirstLeftSecondRight => match ver {
                VerticalPairOrder::FirstTopSecondBottom => {
                    Self::FirstAboveAndLeft
                }
                VerticalPairOrder::FirstBottomSecondTop => {
                    Self::FirstBelowAndLeft
                }
            },
            HorizontalPairOrder::FirstRightSecondLeft => match ver {
                VerticalPairOrder::FirstTopSecondBottom => {
                    Self::FirstAboveAndRight
                }
                VerticalPairOrder::FirstBottomSecondTop => {
                    Self::FirstBelowAndRight
                }
            },
        }
    }

    pub fn print(self) {
        let mut s = "";

        match self {
            Self::FirstAboveAndLeft => s = "above-left | below-right",
            Self::FirstBelowAndLeft => s = "below-left | above-right",
            Self::FirstAboveAndRight => s = "above-right | below-left",
            Self::FirstBelowAndRight => s = "below-right | above-left",
        }

        println!("{s}");
    }
}

impl HandGesture {
    pub const fn is_thumb_up(self) -> bool {
        matches!(self, Self::ThumbUp)
    }

    pub const fn is_thumb_down(self) -> bool {
        matches!(self, Self::ThumbDown)
    }

    pub const fn get_draw_color(self) -> Rgba {
        if LIGHT_MODE {
            match self {
                Self::Unknown => UNKNOWN_HAND_COLOR,
                Self::Open => OPEN_HAND_COLOR,
                Self::Closed => CLOSED_HAND_COLOR,
                Self::ThumbUp | Self::ThumbDown => THUMB_UP_DOWN_HAND_COLOR,
            }
        }
        else {
            match self {
                Self::Unknown => DARK_UNKNOWN_HAND_COLOR,
                Self::Open => DARK_OPEN_HAND_COLOR,
                Self::Closed => DARK_CLOSED_HAND_COLOR,
                Self::ThumbUp | Self::ThumbDown => {
                    DARK_THUMB_UP_DOWN_HAND_COLOR
                }
            }
        }
    }

    pub fn from(s: &str) -> Self {
        match s {
            "Open_Palm" => Self::Open,
            "Closed_Fist" => Self::Closed,
            "Thumb_Up" => Self::ThumbUp,
            "Thumb_Down" => Self::ThumbDown,
            _ => Self::Unknown,
        }
    }
}

// #[derive(Debug)]
// pub struct Hand {
//     // ?
//     pub gesture: HandGesture,
// }

// #[derive(Default, Debug)]
// pub struct Hands {
//     pub first: Option<Hand>,
//     pub second: Option<Hand>,
// }

#[derive(Debug, Clone, Copy, Default)]
pub struct RawHand {
    pub points: [DVec3; NUM_HAND_VERTICES],
    pub gesture: HandGesture,
}

impl RawHand {
    pub fn get_openness_from(&self, com: DVec3) -> f64 {
        let vert_indices = openness_indices();
        let norm = (vert_indices.len() as f64).recip();

        let reference_dist = self.get_proximity();

        let mut acc = 0.0;

        for idx in vert_indices {
            acc += com.distance(self.points[idx]) * norm;
        }

        acc / reference_dist
    }

    pub fn get_proximity(&self) -> f64 {
        // we get the max of these three distances, because they are
        // "relatively" constant compared to other features of the hand as it
        // moves, and having three distances to use provides a little more
        // stability.

        // TODO(jamie): perhaps these could be weighted towards the greater
        // values?
        let dist_01 = self.points[0].distance(self.points[1]);
        let dist_02 = self.points[1].distance(self.points[2]);
        let dist_03 = self.points[2].distance(self.points[3]);

        f64::max(f64::max(dist_01, dist_02), dist_03)
    }

    pub fn get_pinch_for(&self, finger: Finger) -> f64 {
        const MIN_DIST: f64 = 0.1;

        if matches!(finger, Finger::Thumb) {
            return 0.0;
        }

        let compensation_scalar = match finger {
            Finger::Thumb => 1.0,
            Finger::Index => 3.0,
            Finger::Middle => 3.5,
            Finger::Ring => 3.4,
            Finger::Pinky => 4.0,
        };

        let reference_dist =
            self.points[0].distance(self.points[1]) * compensation_scalar;

        let dist = self.points[THUMB_TIP_VERTEX_INDEX]
            .distance(self.points[finger.index()]);

        1.0 - normalize((dist / reference_dist).clamp(0.0, 1.0), MIN_DIST, 1.0)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RawHandPair {
    pub first: Option<RawHand>,
    pub second: Option<RawHand>,
}

#[derive(Debug, Default)]
pub struct ValidRawHandPair {
    pub first: RawHand,
    pub second: RawHand,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct COMPair {
    pub first: Option<DVec3>,
    pub second: Option<DVec3>,
}

#[derive(Debug, Clone, Copy)]
pub struct RawHandPairCOM {
    pub pair: RawHandPair,
    pub com: COMPair,
}

#[derive(Clone, Copy, Debug)]
pub struct CCUpdateData<'a> {
    pub hands: &'a RawHandPairCOM,
    pub velocities: &'a (f32, f32),
    pub mode_sweep: Option<f64>,
}

impl RawHandPairCOM {
    pub fn get_openness(&self) -> (Option<f64>, Option<f64>) {
        let (mut f, mut s) = (None, None);

        if let Some(hand) = &self.pair.first
            && let Some(com) = self.com.first
        {
            f = Some(hand.get_openness_from(com));
        }

        if let Some(hand) = &self.pair.second
            && let Some(com) = self.com.second
        {
            s = Some(hand.get_openness_from(com));
        }

        (f, s)
    }

    pub fn get_proximity(&self) -> (Option<f64>, Option<f64>) {
        let (mut f, mut s) = (None, None);

        if let Some(hand) = &self.pair.first {
            f = Some(hand.get_proximity());
        }

        if let Some(hand) = &self.pair.second {
            s = Some(hand.get_proximity());
        }

        (f, s)
    }
}

impl Default for RawHandPairCOM {
    fn default() -> Self {
        Self {
            pair: RawHandPair { first: None, second: None },
            com: COMPair { first: None, second: None },
        }
    }
}

impl Updatable for RawHandPairCOM {
    fn update(&mut self, update: &Update) {
        self.com.set_from(&self.pair);
    }
}

impl Drawable for RawHandPairCOM {
    fn draw(&self, draw: &Draw, frame: &Frame) {
        self.pair.draw(draw, frame);
        self.com.draw(draw, frame);
    }
}

fn to_xy_and_depth(v: DVec3, wh: DVec2) -> (DVec2, f64) {
    let norm = dvec2(v.x.clamp(0.0, 1.0), 1.0 - v.y.clamp(0.0, 1.0));
    let off = dvec2(1.0, 1.0);

    (
        (norm * 2.0 - off) * (wh * 0.5),
        map(v.z.clamp(-0.05, -0.01), -0.01, -0.05, 0.04, 1.0),
    )
}

/// Converts a set of `r` (red), `g` (green), and `b` (blue) values
/// values to an HSL value.
///
/// [Source](https://www.rapidtables.com/convert/color/rgb-to-hsl.html)
fn rgb_to_hsl(color: Rgb<f32>) -> (f32, f32, f32) {
    let Rgb { red, green, blue, .. } = color;

    let c_max = red.max(blue.max(green));
    let c_min = red.min(blue.min(green));
    let delta = c_max - c_min;

    let l = (c_max + c_min) * 0.5;

    let s =
        if delta == 0.0 { 0.0 } else { delta / (1.0 - (2.0 * l - 1.0).abs()) };

    let h = 60.0
        * if red > blue && red > green {
            ((green - blue) / delta).rem(6.0)
        }
        else if green > red && green > blue {
            (blue - red) / delta + 2.0
        }
        else if blue > red && blue > green {
            (red - green) / delta + 4.0
        }
        else {
            0.0
        };

    (h, s, l)
}

impl Drawable for RawHandPair {
    fn draw(&self, draw: &Draw, frame: &Frame) {
        let wh = frame.rect().wh().as_f64();

        let width = 8.0;
        let dims = Vec2::splat(width);

        if let Some(first) = &self.first {
            let mut col = first.gesture.get_draw_color();

            for (i, p) in first.points.iter().enumerate() {
                let (point, depth) = to_xy_and_depth(*p, wh);
                if LIGHT_MODE {
                    col.alpha = util::xfer::strong_over(depth) as f32;
                }
                else {
                    col.alpha = depth as f32;
                }

                let scale = if is_outer_vertex(i) { 1.5 } else { 1.0 };

                draw.ellipse()
                    .xy(point.as_f32())
                    .wh(dims * scale)
                    .width(width * scale)
                    .color(col)
                    .finish();

                // draw.text(&format!("{i}"))
                //     .wh(vec2(50.0, 50.0))
                //     .xy(point.as_f32())
                //     .font_size(16)
                //     .color(col)
                //     .finish();
            }
        }

        if let Some(second) = &self.second {
            let mut col = second.gesture.get_draw_color();

            for (i, p) in second.points.iter().enumerate() {
                let (point, depth) = to_xy_and_depth(*p, wh);
                col.alpha = depth as f32;

                let scale = if is_outer_vertex(i) { 1.5 } else { 1.0 };

                draw.ellipse()
                    .xy(point.as_f32())
                    .wh(dims * scale)
                    .width(width * scale)
                    .color(col)
                    .finish();

                // draw.text(&format!("{i}"))
                //     .wh(vec2(50.0, 50.0))
                //     .xy(point.as_f32())
                //     .font_size(16)
                //     .color(col)
                //     .finish();
            }
        }
    }
}

impl COMPair {
    pub fn set_from(&mut self, hand_pair: &RawHandPair) {
        const RECIP: f64 = (NUM_HAND_VERTICES as f64).recip();

        const BOT_W: f64 = 4.5;
        const BT2_W: f64 = 1.5;
        const COMPN: f64 = (NUM_HAND_VERTICES as f64 - (BOT_W + BT2_W))
            / (NUM_HAND_VERTICES - 2) as f64;

        #[rustfmt::skip]
        const VERTEX_WEIGHT: [f64; NUM_HAND_VERTICES] = [
            RECIP * BOT_W, RECIP * BT2_W, RECIP * COMPN,
            RECIP * COMPN, RECIP * COMPN, RECIP * COMPN,
            RECIP * COMPN, RECIP * COMPN, RECIP * COMPN,
            RECIP * COMPN, RECIP * COMPN, RECIP * COMPN,
            RECIP * COMPN, RECIP * COMPN, RECIP * COMPN,
            RECIP * COMPN, RECIP * COMPN, RECIP * COMPN,
            RECIP * COMPN, RECIP * COMPN, RECIP * COMPN,
        ];

        self.first = hand_pair.first.map(|h| {
            h.points
                .iter()
                .zip(VERTEX_WEIGHT.iter())
                .fold(DVec3::ZERO, |acc, (&ele, &weight)| acc + (ele * weight))
        });

        self.second = hand_pair.second.map(|h| {
            h.points
                .iter()
                .zip(VERTEX_WEIGHT.iter())
                .fold(DVec3::ZERO, |acc, (&ele, &weight)| acc + (ele * weight))
        });
    }

    pub fn get_horizontal_order(&self) -> Option<HorizontalPairOrder> {
        // if both hands are available, compare them
        if let Some(first) = &self.first
            && let Some(second) = &self.second
        {
            if first.x <= second.x {
                return Some(HorizontalPairOrder::FirstLeftSecondRight);
            }

            return Some(HorizontalPairOrder::FirstRightSecondLeft);
        }

        // if only the second hand is available, check its location
        if let Some(second) = &self.second {
            if second.x <= 0.5 {
                return Some(HorizontalPairOrder::FirstRightSecondLeft);
            }

            return Some(HorizontalPairOrder::FirstLeftSecondRight);
        }

        // if only the first hand is available, check its location
        if let Some(first) = &self.first {
            if first.x <= 0.5 {
                return Some(HorizontalPairOrder::FirstLeftSecondRight);
            }

            return Some(HorizontalPairOrder::FirstRightSecondLeft);
        }

        None
    }

    pub fn get_vertical_order(&self) -> Option<VerticalPairOrder> {
        // note that the y component is flipped as it is top-down

        // if both hands are available, compare them
        if let Some(first) = &self.first
            && let Some(second) = &self.second
        {
            if first.y > second.y {
                return Some(VerticalPairOrder::FirstBottomSecondTop);
            }

            return Some(VerticalPairOrder::FirstTopSecondBottom);
        }

        // if only the second hand is available, check its location
        if let Some(second) = &self.second {
            if second.y > 0.5 {
                return Some(VerticalPairOrder::FirstTopSecondBottom);
            }

            return Some(VerticalPairOrder::FirstBottomSecondTop);
        }

        // if only the first hand is available, check its location
        if let Some(first) = &self.first {
            if first.y > 0.5 {
                return Some(VerticalPairOrder::FirstBottomSecondTop);
            }

            return Some(VerticalPairOrder::FirstTopSecondBottom);
        }

        None
    }

    fn get_text(&self) -> (String, String) {
        let (mut f, mut s) = (String::from("1"), String::from("2"));

        if let Some(hor) = self.get_horizontal_order() {
            if let Some(ver) = self.get_vertical_order() {
                let order = PairOrder::from(hor, ver);

                match order {
                    PairOrder::FirstAboveAndLeft => {
                        f = String::from("AL");
                        s = String::from("BR");
                    }
                    PairOrder::FirstAboveAndRight => {
                        f = String::from("AR");
                        s = String::from("BL");
                    }
                    PairOrder::FirstBelowAndLeft => {
                        f = String::from("BL");
                        s = String::from("AR");
                    }
                    PairOrder::FirstBelowAndRight => {
                        f = String::from("BR");
                        s = String::from("AL");
                    }
                }
            }
            else {
                match hor {
                    HorizontalPairOrder::FirstLeftSecondRight => {
                        f = String::from("L");
                        s = String::from("R");
                    }
                    HorizontalPairOrder::FirstRightSecondLeft => {
                        f = String::from("R");
                        s = String::from("L");
                    }
                }
            }
        }
        else if let Some(ver) = self.get_vertical_order() {
            match ver {
                VerticalPairOrder::FirstTopSecondBottom => {
                    f = String::from("A");
                    s = String::from("B");
                }
                VerticalPairOrder::FirstBottomSecondTop => {
                    f = String::from("B");
                    s = String::from("A");
                }
            }
        }

        (f, s)
    }
}

impl Drawable for COMPair {
    fn draw(&self, draw: &Draw, frame: &Frame) {
        const WEIGHT_MUL: f32 = 0.1;

        let wh = frame.rect().wh().as_f64();

        let width = 24.0;
        let dims = Vec2::splat(width);

        let (first_txt, second_txt) = self.get_text();
        let col =
            if LIGHT_MODE { DEFAULT_COM_COLOR } else { DARK_DEFAULT_COM_COLOR };

        if let Some(first) = &self.first {
            let (point, _) = to_xy_and_depth(*first, wh);
            let mut p = point.as_f32();

            draw.ellipse()
                .no_fill()
                .stroke_color(col)
                .xy(p)
                .wh(dims)
                .stroke_weight(width * WEIGHT_MUL)
                .finish();

            p.y += 3.0;

            draw.text(&first_txt)
                .wh(vec2(80.0, 50.0))
                .xy(p)
                .font_size(14)
                .color(col)
                .finish();
        }

        if let Some(second) = &self.second {
            let (point, _) = to_xy_and_depth(*second, wh);
            let mut p = point.as_f32();

            draw.ellipse()
                .no_fill()
                .stroke_color(col)
                .xy(p)
                .wh(dims)
                .stroke_weight(width * WEIGHT_MUL)
                .finish();

            p.y += 3.0;

            draw.text(&second_txt)
                .wh(vec2(80.0, 50.0))
                .xy(p)
                .font_size(14)
                .color(col)
                .finish();
        }
    }
}
