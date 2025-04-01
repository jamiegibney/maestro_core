//! Audio processing callback.

use crate::{
    dsp::*,
    prelude::xfer::{s_curve_linear_centre, s_curve_round},
};

use super::*;

const SIGNAL_EPSILON: f64 = MINUS_INFINITY_GAIN / 5.0;

/// The main audio processing callback.
pub fn process(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
    let dsp_start = std::time::Instant::now();

    // This works by breaking down the buffer into smaller discrete blocks.
    // For each block, it first processes incoming note events, which are
    // obtained from the `VoiceHandler`. The block size is set to min({samples
    // remaining in buffer}, `MAX_BLOCK_SIZE`, {next event index - block start
    // index}).

    // has to be extracted here because it is borrowed in the line below
    let audio_is_idle = audio.is_idle();
    let buffer_len = buffer.len_frames();

    // best not to block at all here - if the VoiceHandler lock can't be
    // obtained, then the note events won't be processed for this buffer.
    // let mut note_handler_guard = context.note_handler.try_lock().ok();
    // let mut next_event =
    //     note_handler_guard.as_mut().and_then(|g| g.next_event());
    let mut next_event = audio
        .message_channels
        .borrow()
        .note_event
        .as_ref()
        .and_then(|ch| ch.try_recv().ok());

    let voice_handler = &mut audio.voice_handler;

    // if there is no note event, no active voice, and there was no audio
    // processed in the last frame, most of the signal processing can be
    // skipped.
    if next_event.is_none() && !voice_handler.is_voice_active() && audio_is_idle
    {
        callback_timer(audio);
        return;
    }

    let mut block_start: usize = 0;
    let mut block_end = MAX_BLOCK_SIZE.min(buffer_len);

    // audio generators
    while block_start < buffer_len {
        // first, handle incoming events.
        'events: loop {
            match next_event {
                // if the event is now (or before the block), match
                // the event and handle its voice accordingly.
                Some(event) if (event.timing() as usize) <= block_start => {
                    match event {
                        NoteEvent::NoteOn { note, .. } => {
                            voice_handler.start_voice(
                                note,
                                audio.data.sample_rate.lr(),
                                None,
                            );
                        }
                        NoteEvent::NoteOff { note, .. } => {
                            voice_handler.start_release_for_voice(None, note);
                        }
                    }

                    // then obtain the next event and loop again
                    next_event = audio
                        .message_channels
                        .borrow()
                        .note_event
                        .as_ref()
                        .and_then(|ch| ch.try_recv().ok());
                }
                // if the event exists within this block, set the next block
                // to start at the event and continue processing the block
                Some(event) if (event.timing() as usize) < block_end => {
                    block_end = event.timing() as usize;
                    break 'events;
                }
                _ => break 'events,
            }
        }

        let block_len = block_end - block_start;

        let mut gain = [0.0; MAX_BLOCK_SIZE];
        audio.data.voice_gain.next_block(&mut gain, block_len);

        voice_handler.process_block(buffer, block_start, block_end, gain);

        voice_handler.terminate_finished_voices();

        block_start = block_end;
        block_end = (block_end + MAX_BLOCK_SIZE).min(buffer_len);
    }

    // audio effects/processors
    process_fx(audio, buffer);
    callback_timer(audio);
}

/// Sets the audio callback timer.
fn callback_timer(audio: &AudioModel) {
    // the chance of not being able to acquire the lock is very small here,
    // but because this is the audio thread, it's preferable to not block at
    // all. so if the lock can't be obtained, then the callback_time_elapsed
    // will temporarily not be reset. this won't cause issues in the context
    // of this program.
    if let Ok(mut guard) = audio.data.callback_time_elapsed.try_lock() {
        if guard.elapsed().as_secs_f64() >= 0.0001 {
            *guard = std::time::Instant::now();
        }
    }
}

/// Processes all audio FX.
#[allow(clippy::needless_range_loop)]
fn process_fx(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
    //
}
