use super::hand_types::*;
use super::*;

use nannou_osc::Packet;
use serde_json::{Error, Value};

const JSON_HAND_SCHEMA: &str =
    include_str!("../../../assets/json/hand_schema.json");

pub struct HandParser {
    validator: jsonschema::Validator,
    curr_strings: [String; 4],

    first_hand_buf: [DVec3; NUM_HAND_VERTICES],
    second_hand_buf: [DVec3; NUM_HAND_VERTICES],

    first_hand_gesture: HandGesture,
    second_hand_gesture: HandGesture,

    has_second: bool,
}

impl HandParser {
    pub fn new() -> Self {
        let schema: Value = serde_json::from_str(JSON_HAND_SCHEMA)
            .expect("failed to parse hand schema");
        let validator = jsonschema::Validator::new(&schema)
            .expect("failed to create hand validator");

        Self {
            validator,
            curr_strings: [const { String::new() }; 4],

            first_hand_buf: [DVec3::default(); NUM_HAND_VERTICES],
            second_hand_buf: [DVec3::default(); NUM_HAND_VERTICES],

            first_hand_gesture: HandGesture::default(),
            second_hand_gesture: HandGesture::default(),

            has_second: false,
        }
    }

    pub fn parse_hands(
        &mut self,
        packet: Packet,
    ) -> Result<RawHandPair, String> {
        let msgs = packet.into_msgs();

        for m in msgs {
            let strings: Vec<String> =
                m.args.into_iter().filter_map(|a| a.string()).collect();

            if strings.len() != 4 {
                return Err(format!(
                    "received {} strings, but 4 were expected",
                    strings.len()
                ));
            }

            self.curr_strings
                .iter_mut()
                .zip(strings.into_iter())
                .for_each(|(x, new)| {
                    *x = new.replace(['[', ']', '\''], "");
                });
        }

        if !self.parse_curr_strings() {
            return Err(String::from("mismatch in number of coordinates"));
        }

        Ok(self.construct_hand_pair())
    }

    fn parse_curr_strings(&mut self) -> bool {
        let (mut x_count, mut y_count, mut z_count) = (0, 0, 0);
        let mut x_vals = [0.0; NUM_HAND_VERTICES * 2];
        let mut y_vals = [0.0; NUM_HAND_VERTICES * 2];
        let mut z_vals = [0.0; NUM_HAND_VERTICES * 2];

        for (i, x) in self.curr_strings[0].split(',').enumerate() {
            // dbg!(x);
            let val = x.trim().parse().expect("failed to parse value");
            x_vals[i] = val;
            x_count += 1;
        }

        for (i, y) in self.curr_strings[1].split(',').enumerate() {
            let val = y.trim().parse().expect("failed to parse value");
            y_vals[i] = val;
            y_count += 1;
        }

        for (i, z) in self.curr_strings[2].split(',').enumerate() {
            let val = z.trim().parse().expect("failed to parse value");
            z_vals[i] = val;
            z_count += 1;
        }

        if (x_count + y_count + z_count) % NUM_HAND_VERTICES != 0
            || x_count != y_count
            || x_count != z_count
        {
            println!("mismatch in number of coordinate elements: ({x_count}, {y_count}, {z_count}");
            return false;
        }

        let copy_to_buf = |b: &mut [DVec3; NUM_HAND_VERTICES], off| {
            for (((x, y), z), target) in x_vals
                .iter()
                .skip(off)
                .zip(y_vals.iter().skip(off))
                .zip(z_vals.iter().skip(off))
                .zip(b.iter_mut())
            {
                target.x = *x;
                target.y = *y;
                target.z = *z;
            }
        };

        let gestures: Vec<&str> = self.curr_strings[3].split(',').collect();

        self.has_second = x_count / NUM_HAND_VERTICES > 1;

        copy_to_buf(&mut self.first_hand_buf, 0);
        if !gestures.is_empty() {
            self.first_hand_gesture = HandGesture::from(gestures[0].trim());
        }

        if self.has_second {
            copy_to_buf(&mut self.second_hand_buf, NUM_HAND_VERTICES);

            if gestures.len() > 1 {
                self.second_hand_gesture =
                    HandGesture::from(gestures[1].trim());
            }
        }

        true
    }

    fn construct_hand_pair(&self) -> RawHandPair {
        let mut first = RawHand::default();
        first.points.copy_from_slice(&self.first_hand_buf);
        first.gesture = self.first_hand_gesture;

        let mut second = RawHand::default();
        if self.has_second {
            second.points.copy_from_slice(&self.second_hand_buf);
            second.gesture = self.second_hand_gesture;
        }

        RawHandPair {
            first: Some(first),
            second: self.has_second.then_some(second),
        }
    }

    fn validate(&self, json: &Value) -> bool {
        self.validator.is_valid(json)
    }
}
