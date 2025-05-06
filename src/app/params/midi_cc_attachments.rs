use std::collections::HashMap;

use attachment::{MIDICCAttachment, MIDICCFn, MIDICCPredicate, MIDICCSize};
use hands::hand_types::{CCUpdateData, Finger};
use midi_types::MIDICCIndex;
use state::ParameterState;

use super::*;

const MOVE_SMOOTHING_TIME: f32 = 0.05;
const PINCH_SMOOTHING_TIME: f32 = 0.1;
const OPENNESS_SMOOTHING_TIME: f32 = 0.15;
const PROXIMITY_SMOOTHING_TIME: f32 = 0.125;
const VELOCITY_SMOOTHING_TIME: f32 = 0.2;

/// See [`DEFAULT_MIDI_CC_UPDATE_THRESHOLD`]
const MOVE_UPDATE_THRESHOLD: f32 = 0.00001;
const VELOCITY_UPDATE_THRESHOLD: f32 = 0.00025;
const PINCH_UPDATE_THRESHOLD: f32 = 0.001;
const OPENNESS_UPDATE_THRESHOLD: f32 = 0.001;

fn add<'a>(
    hm: &'a mut HashMap<MIDICCIndex, MIDICCAttachment>,
    channel: u8,
    cc: u8,
    name: &str,
    callback: MIDICCFn,
    predicate: MIDICCPredicate,
) -> &'a mut MIDICCAttachment {
    let idx = MIDICCIndex::new(channel, cc);
    let attachment = MIDICCAttachment::new(
        name,
        callback,
        predicate,
        None,
        MIDICCSize::CC7Bit,
        DEFAULT_MIDI_CC_UPDATE_THRESHOLD,
    );

    hm.insert(idx, attachment);

    unsafe { hm.get_mut(&idx).unwrap_unchecked() }
}

#[allow(
    clippy::too_many_lines, clippy::complexity, clippy::cognitive_complexity
)]
pub fn build_midi_cc_attachments() -> HashMap<MIDICCIndex, MIDICCAttachment> {
    let mut hm = HashMap::new();

    // First hand

    add(
        &mut hm,
        MIDI_CHANNEL_1,
        MIDI_CC14_0,
        "First hand x-pos",
        |values: &CCUpdateData, cc: &mut f32| {
            if let Some(com) = &values.hands.com.first {
                *cc = com.x as f32;
            }
        },
        |state: &ParameterState| true,
    )
    .with_smoothing_time(MOVE_SMOOTHING_TIME)
    .with_size(MIDICCSize::CC14Bit)
    .with_update_threshold(MOVE_UPDATE_THRESHOLD);

    add(
        &mut hm,
        MIDI_CHANNEL_1,
        MIDI_CC14_1,
        "First hand y-pos",
        |values: &CCUpdateData, cc: &mut f32| {
            if let Some(com) = &values.hands.com.first {
                *cc = 1.0 - (com.y as f32);
            }
        },
        |state: &ParameterState| true,
    )
    .with_smoothing_time(MOVE_SMOOTHING_TIME)
    .with_size(MIDICCSize::CC14Bit)
    .with_update_threshold(MOVE_UPDATE_THRESHOLD);

    add(
        &mut hm,
        MIDI_CHANNEL_1,
        MIDI_CC_2,
        "First hand openness",
        |values: &CCUpdateData, cc: &mut f32| {
            if let Some(hand) = &values.hands.pair.first
                && let Some(com) = values.hands.com.first
            {
                let openness =
                    map(hand.get_openness_from(com), 0.72, 2.0, 0.0, 1.0)
                        .clamp(0.0, 1.0);
                *cc = openness as f32;
            }
        },
        |state: &ParameterState| true,
    )
    .with_update_threshold(OPENNESS_UPDATE_THRESHOLD)
    .with_smoothing_time(OPENNESS_SMOOTHING_TIME);

    add(
        &mut hm,
        MIDI_CHANNEL_1,
        MIDI_CC_3,
        "First hand proximity",
        |values: &CCUpdateData, cc: &mut f32| {
            if let Some(hand) = &values.hands.pair.first {
                let proximity = map(hand.get_proximity(), 0.03, 0.06, 0.0, 1.0)
                    .clamp(0.0, 1.0);
                *cc = proximity as f32;
            }
        },
        |state: &ParameterState| true,
    )
    .with_smoothing_time(PROXIMITY_SMOOTHING_TIME);

    add(
        &mut hm,
        MIDI_CHANNEL_1,
        MIDI_CC_4,
        "First hand index finger pinch",
        |values: &CCUpdateData, cc: &mut f32| {
            if let Some(hand) = &values.hands.pair.first {
                let pinch = hand.get_pinch_for(Finger::Index);
                *cc = pinch as f32;
            }
        },
        |state: &ParameterState| true,
    )
    .with_update_threshold(PINCH_UPDATE_THRESHOLD)
    .with_smoothing_time(PINCH_SMOOTHING_TIME);

    add(
        &mut hm,
        MIDI_CHANNEL_1,
        MIDI_CC_5,
        "First hand middle finger pinch",
        |values: &CCUpdateData, cc: &mut f32| {
            if let Some(hand) = &values.hands.pair.first {
                let pinch = hand.get_pinch_for(Finger::Middle);
                *cc = pinch as f32;
            }
        },
        |state: &ParameterState| true,
    )
    .with_update_threshold(PINCH_UPDATE_THRESHOLD)
    .with_smoothing_time(PINCH_SMOOTHING_TIME);

    add(
        &mut hm,
        MIDI_CHANNEL_1,
        MIDI_CC_6,
        "First hand ring finger pinch",
        |values: &CCUpdateData, cc: &mut f32| {
            if let Some(hand) = &values.hands.pair.first {
                let pinch = hand.get_pinch_for(Finger::Ring);
                *cc = pinch as f32;
            }
        },
        |state: &ParameterState| true,
    )
    .with_smoothing_time(PINCH_SMOOTHING_TIME);

    add(
        &mut hm,
        MIDI_CHANNEL_1,
        MIDI_CC_7,
        "First hand pinky finger pinch",
        |values: &CCUpdateData, cc: &mut f32| {
            if let Some(hand) = &values.hands.pair.first {
                let pinch = hand.get_pinch_for(Finger::Pinky);
                *cc = pinch as f32;
            }
        },
        |state: &ParameterState| true,
    )
    .with_smoothing_time(PINCH_SMOOTHING_TIME);

    add(
        &mut hm,
        MIDI_CHANNEL_1,
        MIDI_CC_8,
        "First hand velocity",
        |values: &CCUpdateData, cc: &mut f32| {
            if let Some(hand) = &values.hands.pair.first {
                *cc = values.velocities.0;
            }
        },
        |state: &ParameterState| true,
    )
    .with_update_threshold(VELOCITY_UPDATE_THRESHOLD)
    .with_smoothing_time(VELOCITY_SMOOTHING_TIME);

    // *** *** *** *** *** //

    add(
        &mut hm,
        MIDI_CHANNEL_3,
        MIDI_CC_0,
        "Mode sweep",
        |values: &CCUpdateData, cc: &mut f32| {
            use std::f32::consts::PI;
            *cc = values.mode_sweep.map_or(0.0, |x| f32::sin(x as f32 * PI));
        },
        |state: &ParameterState| true,
    )
    .with_smoothing_time(0.02)
    .with_update_threshold(0.02);

    // // Second hand
    //
    // add(
    //     &mut hm,
    //     MIDI_CHANNEL_2,
    //     MIDI_CC14_0,
    //     "Second hand x-pos",
    //     |values: &SignificantHandValues, cc: &mut f32| {
    //         if let Some(com) = &values.hands.com.second {
    //             *cc = com.x as f32;
    //         }
    //     },
    //     |state: &ParameterState| true,
    // )
    // .with_smoothing_time(MOVE_SMOOTHING_TIME)
    // .with_size(MIDICCSize::CC14Bit)
    // .with_update_threshold(MOVE_UPDATE_THRESHOLD);
    //
    // add(
    //     &mut hm,
    //     MIDI_CHANNEL_2,
    //     MIDI_CC14_1,
    //     "Second hand y-pos",
    //     |values: &SignificantHandValues, cc: &mut f32| {
    //         if let Some(com) = &values.hands.com.second {
    //             *cc = 1.0 - (com.y as f32);
    //         }
    //     },
    //     |state: &ParameterState| true,
    // )
    // .with_smoothing_time(MOVE_SMOOTHING_TIME)
    // .with_size(MIDICCSize::CC14Bit)
    // .with_update_threshold(MOVE_UPDATE_THRESHOLD);
    //
    // add(
    //     &mut hm,
    //     MIDI_CHANNEL_2,
    //     MIDI_CC_2,
    //     "Second hand openness",
    //     |values: &SignificantHandValues, cc: &mut f32| {
    //         if let Some(hand) = &values.hands.pair.second
    //             && let Some(com) = values.hands.com.second
    //         {
    //             let openness =
    //                 map(hand.get_openness_from(com), 1.0, 3.0, 0.0, 1.0)
    //                     .clamp(0.0, 1.0);
    //             *cc = openness as f32;
    //         }
    //     },
    //     |state: &ParameterState| true,
    // )
    // .with_smoothing_time(OPENNESS_SMOOTHING_TIME);
    //
    // add(
    //     &mut hm,
    //     MIDI_CHANNEL_2,
    //     MIDI_CC_3,
    //     "Second hand proximity",
    //     |values: &SignificantHandValues, cc: &mut f32| {
    //         if let Some(hand) = &values.hands.pair.second {
    //             let proximity = map(hand.get_proximity(), 0.03, 0.12, 0.0,
    // 1.0)                 .clamp(0.0, 1.0);
    //             *cc = proximity as f32;
    //         }
    //     },
    //     |state: &ParameterState| true,
    // )
    // .with_smoothing_time(PROXIMITY_SMOOTHING_TIME);
    //
    // add(
    //     &mut hm,
    //     MIDI_CHANNEL_2,
    //     MIDI_CC_4,
    //     "Second hand index finger pinch",
    //     |values: &SignificantHandValues, cc: &mut f32| {
    //         if let Some(hand) = &values.hands.pair.second {
    //             let pinch = hand.get_pinch_for(Finger::Index);
    //             *cc = pinch as f32;
    //         }
    //     },
    //     |state: &ParameterState| true,
    // )
    // .with_smoothing_time(PINCH_SMOOTHING_TIME);
    //
    // add(
    //     &mut hm,
    //     MIDI_CHANNEL_2,
    //     MIDI_CC_5,
    //     "Second hand middle finger pinch",
    //     |values: &SignificantHandValues, cc: &mut f32| {
    //         if let Some(hand) = &values.hands.pair.second {
    //             let pinch = hand.get_pinch_for(Finger::Middle);
    //             *cc = pinch as f32;
    //         }
    //     },
    //     |state: &ParameterState| true,
    // )
    // .with_smoothing_time(PINCH_SMOOTHING_TIME);
    //
    // add(
    //     &mut hm,
    //     MIDI_CHANNEL_2,
    //     MIDI_CC_6,
    //     "Second hand velocity",
    //     |values: &SignificantHandValues, cc: &mut f32| {
    //         if let Some(hand) = &values.hands.pair.second {
    //             *cc = values.velocities.1;
    //         }
    //     },
    //     |state: &ParameterState| true,
    // )
    // .with_update_threshold(VELOCITY_UPDATE_THRESHOLD)
    // .with_smoothing_time(VELOCITY_SMOOTHING_TIME);

    hm
}
