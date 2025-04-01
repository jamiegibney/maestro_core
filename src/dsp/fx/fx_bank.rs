use std::collections::VecDeque;

use super::*;

pub const MAX_NUM_FX_PER_BANK: usize = 16;

#[derive(Clone, Copy, Debug)]
pub enum FXBankError {
    MultipleInstancesFound,
    UnknownIndex,
}

#[derive(Clone, Debug)]
struct FXProcessor {
    pub id: u32,
    pub processor: Box<dyn Effect>,
}

#[derive(Clone, Debug)]
pub struct FXBank {
    processors: [Option<FXProcessor>; MAX_NUM_FX_PER_BANK],
    sample_rate: f64,
    id_counter: u32,
}

impl FXBank {
    pub const fn new(sample_rate: f64) -> Self {
        Self {
            processors: [const { None }; MAX_NUM_FX_PER_BANK],
            sample_rate,
            id_counter: 0,
        }
    }

    pub fn get_ids(&self) -> [Option<u32>; MAX_NUM_FX_PER_BANK] {
        let mut output = [const { None }; MAX_NUM_FX_PER_BANK];

        for i in 0..MAX_NUM_FX_PER_BANK {
            output[i] = self.get_id_for(i);
        }

        output
    }

    pub fn get_identifiers(&self) -> [Option<&str>; MAX_NUM_FX_PER_BANK] {
        let mut output = [const { None }; MAX_NUM_FX_PER_BANK];

        for i in 0..MAX_NUM_FX_PER_BANK {
            output[i] = self.get_identifer_for(i);
        }

        output
    }

    /// # Panics
    ///
    /// Panics if `idx >= MAX_NUM_FX_PER_BANK`.
    pub fn access_fx_mut(&mut self, idx: usize) -> Option<&mut Box<dyn Effect>> {
        assert!(idx < MAX_NUM_FX_PER_BANK, "provided index ({idx}) exceeded FX bounds (max {MAX_NUM_FX_PER_BANK})");
        self.processors[idx].as_mut().map(|p| &mut p.processor)
    }

    pub fn access_fx_by_id_mut(&mut self, id: u32) -> Option<&mut Box<dyn Effect>> {
        self.processors.iter_mut().flatten().find(|p| p.id == id).map(|p| &mut p.processor)
    }

    /// # Panics
    ///
    /// Panics if `idx >= MAX_NUM_FX_PER_BANK`.
    pub fn get_id_for(&self, idx: usize) -> Option<u32> {
        assert!(idx < MAX_NUM_FX_PER_BANK, "provided index ({idx}) exceeded FX bounds (max {MAX_NUM_FX_PER_BANK})");
        self.processors[idx].as_ref().map(|p| p.id)
    }

    /// # Panics
    ///
    /// Panics if `idx >= MAX_NUM_FX_PER_BANK`.
    pub fn get_identifer_for(&self, idx: usize) -> Option<&str> {
        assert!(idx < MAX_NUM_FX_PER_BANK, "provided index ({idx}) exceeded FX bounds (max {MAX_NUM_FX_PER_BANK})");
        self.processors[idx].as_ref().map(|p| p.processor.get_identifier())
    }

    /// Returns the number of currently-active effects.
    pub fn num_active_fx(&self) -> usize {
        self.processors.iter().filter(|&x| x.is_some()).count()
    }

    /// # Errors
    ///
    /// If no slots are available in the FX bank, this method will return the
    /// provided effect as `Err`.
    ///
    /// # Panics
    ///
    /// Panics if the sample rate of `effect` does not match the sample rate of
    /// the FX bank.
    pub fn push_effect<E: Effect + 'static>(
        &mut self,
        effect: E,
    ) -> Result<(), E> {
        assert!(
            eps_eq(self.sample_rate, effect.get_sample_rate()), 
            "mismatched sample rate: cannot push effect with different sample rate to FX bank"
        );

        let curr_id = self.id_counter;

        if let Some(slot) = self.get_next_slot() {
            slot.replace(FXProcessor {
                id: curr_id,
                processor: Box::new(effect),
            });
        }
        else {
            return Err(effect);
        }

        self.id_counter.wrapping_add(1);

        self.collapse_fx();

        Ok(())
    }

    fn get_next_slot(&mut self) -> Option<&mut Option<FXProcessor>> {
        self.processors.iter_mut().find(|x| x.is_none())
    }

    fn collapse_fx(&mut self) {
        let num_active = self.num_active_fx();

        if (num_active == 0) {
            return;
        }

        let mut num_found = 0;
        let mut available_slots: VecDeque<usize> = VecDeque::new();

        for i in 0..MAX_NUM_FX_PER_BANK {
            // we don't need to re-order if we've found all active processors.
            if num_found == num_active {
                return;
            }

            // if we've found an empty slot whilst active processors still
            // remain, we add the slot as available for a swap.
            if self.processors[i].is_none() {
                available_slots.push_back(i);
                continue;
            }

            // if we've found an active processor and there is an empty slot at
            // the front of the queue, we pop the slot and swap the elements,
            // and then mark the current slot as available
            if let Some(available) = available_slots.pop_front() {
                self.processors.swap(i, available);
                available_slots.push_back(i);
            }

            num_found += 1;
        }
    }
}

impl Effect for FXBank {
    fn process_mono(&mut self, input: f64, channel_idx: usize) -> f64 {
        let mut out = input;

        for p in self.processors.iter_mut().flatten() {
            out = p.processor.process_mono(out, channel_idx);
        }

        out
    }

    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let mut out = (in_l, in_r);

        for p in self.processors.iter_mut().flatten() {
            out = p.processor.process_stereo(out.0, out.1);
        }

        out
    }

    fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }

    fn get_identifier(&self) -> &str {
        "fx_bank"
    }
}

