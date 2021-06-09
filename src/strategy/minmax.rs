//! Implementation of the min max algorithm.
use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use std::fmt;
use rayon::prelude::*;

/// Min-Max algorithm with a given recursion depth.
pub struct MinMax(pub u8);

impl Strategy for MinMax {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        state.movements()
                .collect::<Vec<Movement>>()
                .par_iter()
                .map(|coup| (coup, -negamax(self.0 - 1, state.play(&coup))))
                .max_by_key(|&(_, val)| val)
                .map(|(res, _)| *res)
    }
}

impl fmt::Display for MinMax {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Min - Max (max level: {})", self.0)
    }
}

fn negamax(profondeur: u8, state: Configuration) -> i8 {
    if profondeur == 0 {
        -state.value()
    } else {
        state.movements()
             .into_iter()
             .map(|coup| -negamax(profondeur - 1, state.play(&coup)))
             .max()
             .unwrap_or(-state.value())
    }
}

fn minimax(profondeur: u8, maximizing: bool, state: Configuration) -> i8 {
    if profondeur == 0 {
        if maximizing {
            -state.value()
        } else {
            state.value()
        }
    } else { // Séquentiel sur la fin
        state.movements()
            .into_iter()
            .map(|coup| minimax(profondeur - 1, ! maximizing, state.play(&coup)))
            .max_by_key(|&val| {
                if maximizing {
                    val
                } else {
                    -val
                }
            })
            .unwrap_or(if maximizing {
                            -state.value() 
                        } else { 
                            state.value()
            })
    }
}

/// Anytime min max algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn min_max_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        movement.store(MinMax(depth).compute_next_move(state));
    }
}
