# Maestro core

> [!IMPORTANT]
> This repository was archived on 24/05/2025 as part of a University
> submission. It contains a prototype version of "Maestro" — an application
> which enables hand gestures to output OSC and MIDI information.
>
> The gesture recognition is performed via Google's Media Pipe, which is hosted
> within Touch Designer. Touch Designer emits OSC messages, which are received
> by `maestro_core` for parsing and processing. `maestro_core` can emit OSC and
> MIDI data to the system, which can be utilised to control various
> applications.
>
> As mentioned, this is a prototype. The implementation is messy, inefficient,
> and far from complete.

> [!NOTE]
> Please note that there are many modules/source files which are unused,
> because this project was initially a fork of [this
> repository](https://github.com/jamiegibney/creative_coding_project). Most of
> the implementation is in the following modules:
> - `src/app/hands/`
> - `src/app/midi/`
> - `src/app/osc/`
> - `src/app/params/`

