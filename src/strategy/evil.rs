//! Alpha - Beta algorithm.
use std::fmt;

use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;

/// Anytime alpha beta algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn alpha_beta_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        let chosen_movement = Evil(depth).compute_next_move(state);
        movement.store(chosen_movement);
    }
}

/// Alpha - Beta algorithm with given maximum number of recursions.
pub struct Evil(pub u8);

impl fmt::Display for Evil {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Evil (max level: {})", self.0)
    }
}

impl Strategy for Evil {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        alphabeta_sorted(self.0, -127, 127, *state).0
    }
}


fn negamax2(profondeur: u8, mut alpha: i8, beta: i8, state: Configuration) -> (i8, i8, i8) {
    if profondeur == 0 || state.movements().next().is_none() {
        (-state.value(), alpha, beta)
    } else {
        let mut best_val = -127;
        for coup in state.movements().into_iter() {
            let val = -negamax2(profondeur - 1, -beta, -alpha, state.play(&coup)).0;
            if val > best_val {
                best_val = val;
                if best_val > alpha {
                    alpha = best_val;
                }
                if alpha >= beta {
                    break;
                }
            }
        }
        (best_val, alpha, beta)
    }
}

fn alphabeta(profondeur: u8, mut alpha: i8, mut beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 || state.movements().next().is_none() {
        (None, -state.value())
    } else {
        let mut best_move = None;
        let mut best_val = -127;
        for coup in state.movements().into_iter() {
            let (_, val0) = alphabeta(profondeur - 1, -beta, -alpha, state.play(&coup));
            let val = -val0;
            if val > best_val {
                best_val = val;
                best_move = Some(coup);
                if best_val > alpha {
                    alpha = best_val;
                }
                if alpha >= beta { // or val ?
                    break; // Problème pr paralléliser
                }
            }
        }
        return (best_move, best_val);
    }
}

fn alphabeta_sorted(profondeur: u8, mut alpha: i8, mut beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 {
        (None, -state.value())
    } else {
        let mut best_move = None;
        let mut best_val = -127;
        let mut mouvements_ordonnes = state.movements().collect::<Vec<Movement>>();
        if profondeur >= 4 {
            mouvements_ordonnes.sort_by_key(|&coup| -alphabeta(2, alpha, beta, state.play(&coup)).1);
        }
        for coup in mouvements_ordonnes.into_iter() {
            let (_, val0) = alphabeta(profondeur - 1, -beta, -alpha, state.play(&coup));
            let val = -val0;
            if val > best_val {
                best_val = val;
                best_move = Some(coup);
                if best_val > alpha {
                    alpha = best_val;
                }
                if val >= beta {
                    break; // Problème pr paralléliser
                }
            }
        }
        return (best_move, best_val);
    }
}

fn alphabeta_iteratif(profondeur: u8, mut alpha: i8, mut beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    let mut best_move = None;
    let mut alpha = -127;
    for i in 1..(profondeur+1) {
        let (tmp_coup, tmp_alpha) = alphabeta(i, alpha, -alpha, state);
        if i8::abs(tmp_alpha) >= i8::abs(alpha) {
            alpha = -1 * i8::abs(tmp_alpha);
        }
        best_move = tmp_coup;
    }
    (best_move, alpha)

}

fn alphabeta_par(profondeur: u8, mut alpha: i8, mut beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 {
        (None, -state.value())
    } else {
        let mut best_move = None;
        let mut best_val = -127;
        for coup in state.movements().into_iter() {
            let (_, val0) = alphabeta(profondeur - 1, -beta, -alpha, state.play(&coup));
            let val = -val0;
            if val > best_val {
                best_val = val;
                best_move = Some(coup);
                if best_val > alpha {
                    alpha = best_val;
                }
                if val >= beta {
                    break; // Problème pr paralléliser
                }
            }
        }
        return (best_move, best_val);
    }
}