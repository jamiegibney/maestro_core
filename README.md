# Maestro core

## TODO

- [ ] Implement real-time EME request
- [ ] Test EME OSC output
- [ ] EME request sender channel

### Major

- [x] Gesture information receiver

OSC receiver for hand gesture data from Touch Designer

- [x] MIDI representation

Representation of MIDI data with functions for converting to MIDI bytes, and
methods for sending to MIDI ports

- [x] EME requests

Representation, validation, and serialisation of EME requests.

- [x] Timer thread

Type for spawning a thread which calls periodically calls a job, with
relatively precise timing.

- [x] MIDI sender thread

Timer thread which periodically sends a collection of MIDI data to a MIDI port.

- [x] OSC sender thread

Timer thread which periodically sends OSC requests to the EME **if** there is
an EME request available.

- [x] Parameter management

System which transforms hand gesture information into a collection of MIDI and
EME requests. Posts EME requests via an `mpsc` channel, and MIDI request via a
triple buffer?

There are two approaches to updating MIDI information for Ableton.

The first method is to collect *all* MIDI CC/notes and their current values,
and send all of them every time the MIDI timer thread calls its send callback.
This could use a triple buffer and is relative easy to implement, but does
require sending a lot of requests, and that Ableton has to consume a large
number of MIDI changes at a high rate.

The second method is to collect only the *deltas* for all MIDI CC/notes. This
means that all default values need to be sent initially, and then whenever a
particular MIDI value is update it is "marked" to be sent to the MIDI port.
This potentially means that less MIDI events need to be sent each time the MIDI
timer thread calls its callback, and that Ableton has less MIDI data to consume
overall. But it requires a bit more work within the parameter system.

It is important to consider that parameters will be continuously smoothed — but
not necessarily all parameters will be used at one time. So perhaps the second
method could be employed — and if a large number of parameters need to be
updated, then a large number get marked and the MIDI send thread just sends all
of them.

- [x] Parameter update/interpolation thread

The parameter system will likely need to smooth/interpolate its parameters over
time in order to ensure smooth changes in MIDI/OSC data. This means that all
parameters need to be updated at a relatively high rate — ideally *at least*
the rate of the MIDI sending thread.

This thread will keep track of a delta time as to smooth all parameters over
time, and will transform the information from the gesture information into each
MIDI/OSC parameter. It marks all parameters which are modified, and then pushes
all of them into a queue or buffer which is then picked up by the MIDI/OSC
sender thread. A `mpsc` channel or triple buffer could be use — but as the
changes would be consumed, it probably makes sense to use a channel.

## Structure

### Program receivers

- Gesture data via OSC -> `OSCReceiver`

### Program senders

- Parameter data via MIDI -> `MIDISender`
- EME data via OSC -> `EMERequestOSCSender`

### Internal channels

- Gesture information from "hands" to "parameters" (triple buffer)
- MIDI information from "parameter system" to "midi sender" (mpsc)
- EME information from "parameter system" to "osc sender" (mpsc)
