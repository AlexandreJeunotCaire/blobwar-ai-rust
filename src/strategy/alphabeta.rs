//! Alpha - Beta algorithm.
use std::fmt;

use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use rayon::prelude::*;
use std::collections::HashMap;

/// Anytime alpha beta algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn alpha_beta_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        let chosen_movement = AlphaBeta(depth).compute_next_move(state);
        movement.store(chosen_movement);
    }
}

/// Alpha - Beta algorithm with given maximum number of recursions.
pub struct AlphaBeta(pub u8);

impl fmt::Display for AlphaBeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Alpha - Beta (max level: {})", self.0)
    }
}

impl Strategy for AlphaBeta {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        //alphabeta(self.0, -127, 127, *state).0

        /*
        let score_courant = state.value();
        alphabeta_aspiration(self.0, score_courant - 30, score_courant + 30, *state).0
        */
        
        //alphabeta_sorted(self.0, -127, 127, *state).0
        
        //alphabeta_pvs(self.0, -127, 127, *state).0

        //alphabeta_par(self.0, *state)
        
        //alphabeta_par_classic_with_score(self.0, -127, 127, *state).0
        
        //alphabeta_par2(self.0, -127, 127, *state).0

        //alphabeta_par_aspiration(self.0, *state)

        //alphabeta_par_pvs(self.0, *state)
        
        alphabeta_par_pvs2(self.0, -127, 127, *state).0

        //alphabeta_par_pvs_double_depth_par(self.0, -127, 127, *state).0

        //alphabeta_par_infinite(self.0, -127, 127, *state).0
    }
}

fn alphabeta(profondeur: u8, mut alpha: i8, beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        (None, -state.value())
    } else {
        let mut best_move = None;
        let mut best_val = -127;
        
        for coup in state.movements() {
            let val = -alphabeta(profondeur - 1, -beta, -alpha, state.play(&coup)).1;
            if val > best_val {
                best_val = val;
                best_move = Some(coup);
                if best_val > alpha {
                    alpha = best_val;
                    if alpha >= beta { // or val ?
                        break; // Problème pr paralléliser
                    }
                }
            }
        }
        (best_move, best_val)
    }
}

fn alphabeta_aspiration(profondeur: u8, mut alpha: i8, beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        (None, -state.value())
    } else {
        let mut best_move = None;
        let mut best_val = -127;
        let score_courant = state.value();
        for coup in state.movements() {
            let val = -alphabeta(profondeur - 1, alpha, beta, state.play(&coup)).1;
            if val > best_val {
                best_val = val;
                best_move = Some(coup);
                if best_val > alpha {
                    alpha = best_val;
                    if alpha >= beta { // or val ?
                        break; // Problème pr paralléliser
                    }
                }
            }
        }
        if best_move.is_none() || best_val >= score_courant + 30 || best_val <= score_courant + 30 {
            alphabeta(profondeur, -127, 127, state)
        } else {
        (best_move, best_val)
        }
    }
}

fn alphabeta_pvs(profondeur: u8, mut alpha: i8, mut beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        (None, -state.value())
    } else {
        let mut best_move = None;
        let mut best_val = - 127;
        for (i, coup) in state.movements().enumerate() {
            let mut score;
            if i == 0 {
                score = -alphabeta_pvs(profondeur - 1, -beta, -alpha, state.play(&coup)).1;
            } else {
                score = -alphabeta_pvs(profondeur - 1, -alpha - 1, -alpha, state.play(&coup)).1;
                if alpha < score && score < beta {
                    score = -alphabeta_pvs(profondeur - 1, -beta, -score, state.play(&coup)).1;
                }
            }
            if score > best_val {
                best_val = score;
                best_move = Some(coup);
                if best_val > alpha {
                    alpha = best_val;
                    if alpha >= beta {
                        break;
                    }
                }
            }
        }
        (best_move, alpha)
    }
}

fn alphabeta_par(profondeur: u8, state: Configuration) -> Option<Movement> {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        None
    } else {
        state.movements()
                .collect::<Vec<Movement>>()
                .par_iter()
                .map(|coup| (coup, -alphabeta(profondeur - 1, -127, 127, state.play(&coup)).1))
                .max_by_key(|&(_, val)| val)
                .map(|(res, _)| *res)
    }
}

fn alphabeta_par_classic_with_score(profondeur: u8, mut alpha: i8, mut beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        (None, -state.value())
    } else {
        state.movements()
        .collect::<Vec<Movement>>()
        .par_iter()
        .map(|coup| (Some(*coup), -alphabeta_pvs(profondeur - 1, alpha, beta, state.play(&coup)).1))
        .max_by_key(|&(_, val)| val)
        .unwrap_or((None, -state.value()))
    }
}

fn alphabeta_par_infinite(profondeur: u8, mut alpha: i8, mut beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        (None, -state.value())
    } else {
        state.movements()
        .collect::<Vec<Movement>>()
        .par_iter()
        .map(|coup| (Some(*coup), -alphabeta_par_infinite(profondeur - 1, alpha, beta, state.play(&coup)).1))
        .max_by_key(|&(_, val)| val)
        .unwrap_or((None, -state.value()))
    }
}

fn alphabeta_par_pvs(profondeur: u8, state: Configuration) -> Option<Movement> {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        None
    } else {
        state.movements()
                .collect::<Vec<Movement>>()
                .par_iter()
                .map(|coup| (coup, -alphabeta_pvs(profondeur - 1, -127, 127, state.play(&coup)).1))
                .max_by_key(|&(_, val)| val)
                .map(|(res, _)| *res)
    }
}

fn alphabeta_par_pvs2(profondeur: u8, mut alpha: i8, beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        (None, -state.value())
    } else {
        let (coup, val) = state.movements()
             .collect::<Vec<Movement>>()
             .par_iter()
             .try_fold(|| -> (Option<Movement>, i8) { (None, -127 as i8) },
                       |(best_move, best_val), coup| {
                let mut alpha2 = alpha;
                let vala = -alphabeta_pvs(profondeur - 1, -beta, -alpha2, state.play(&coup)).1;
                let mut tmp_bc = best_move;
                let mut tmp_bv = best_val;
                if vala > best_val {
                    tmp_bv = vala;
                    tmp_bc = Some(*coup);
                    if tmp_bv > alpha {
                        alpha2 = tmp_bv;
                        if alpha2 >= beta {
                            return Err((tmp_bc, tmp_bv));
                        }
                    }
                    return Ok((tmp_bc, tmp_bv));
                } else {
                    return Ok((best_move, best_val));
                }
             })
             .reduce(|| -> Result<(Option<Movement>, i8), (Option<Movement>, i8)> { Ok((None, 0)) }, |a, b| {
                 let (coup1, val1) = a.unwrap();
                 let (coup2, val2) = b.unwrap();
                 if coup1.is_none() {
                     return b;
                 } else if coup2.is_none() {
                     return a;
                 }
                 if val1 > val2 { a } else { b }
             })
             .unwrap_or((None, -state.value()));
        (coup, val)
    }
}

fn alphabeta_par_pvs_double_depth_par(profondeur: u8, mut alpha: i8, beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        (None, -state.value())
    } else {
        let (coup, val) = state.movements()
             .collect::<Vec<Movement>>()
             .par_iter()
             .try_fold(|| -> (Option<Movement>, i8) { (None, -127 as i8) },
                       |(best_move, best_val), coup| {
                let mut alpha2 = alpha;
                let vala = -alphabeta_par_classic_with_score(profondeur - 1, -beta, -alpha2, state.play(&coup)).1;
                let mut tmp_bc = best_move;
                let mut tmp_bv = best_val;
                if vala > best_val {
                    tmp_bv = vala;
                    tmp_bc = Some(*coup);
                    if tmp_bv > alpha {
                        alpha2 = tmp_bv;
                        if alpha2 >= beta {
                            return Err((tmp_bc, tmp_bv));
                        }
                    }
                    return Ok((tmp_bc, tmp_bv));
                } else {
                    return Ok((best_move, best_val));
                }
             })
             .reduce(|| -> Result<(Option<Movement>, i8), (Option<Movement>, i8)> { Ok((None, 0)) }, |a, b| {
                 let (coup1, val1) = a.unwrap();
                 let (coup2, val2) = b.unwrap();
                 if coup1.is_none() {
                     return b;
                 } else if coup2.is_none() {
                     return a;
                 }
                 if val1 > val2 { a } else { b }
             })
             .unwrap_or((None, -state.value()));
        (coup, val)
    }
}

fn alphabeta_par_aspiration(profondeur: u8, state: Configuration) -> Option<Movement> {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        None
    } else {
        let score_courant = state.value();
        let coup = state.movements()
                .collect::<Vec<Movement>>()
                .par_iter()
                .map(|coup| (coup, -alphabeta(profondeur - 1, score_courant - 30, score_courant + 30, state.play(&coup)).1))
                .max_by_key(|&(_, val)| val)
                .map(|(res, _)| *res);
        if coup.is_none() {
            alphabeta_par(profondeur, state)
        } else {
            coup
        }
    }
}

fn alphabeta_par2(profondeur: u8, mut alpha: i8, beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        (None, -state.value())
    } else {
        let (coup, val) = state.movements()
             .collect::<Vec<Movement>>()
             .par_iter()
             .try_fold(|| -> (Option<Movement>, i8) { (None, -127 as i8) },
                       |(best_move, best_val), coup| {
                let mut alpha2 = alpha;
                let vala = -alphabeta(profondeur - 1, -beta, -alpha2, state.play(&coup)).1;
                let mut tmp_bc = best_move;
                let mut tmp_bv = best_val;
                if vala > best_val {
                    tmp_bv = vala;
                    tmp_bc = Some(*coup);
                    if tmp_bv > alpha {
                        alpha2 = tmp_bv;
                        if alpha2 >= beta {
                            return Err((tmp_bc, tmp_bv));
                        }
                    }
                    return Ok((tmp_bc, tmp_bv));
                } else {
                    return Ok((best_move, best_val));
                }
             })
             .reduce(|| -> Result<(Option<Movement>, i8), (Option<Movement>, i8)> { Ok((None, 0)) }, |a, b| {
                 let (coup1, val1) = a.unwrap();
                 let (coup2, val2) = b.unwrap();
                 if coup1.is_none() {
                     return b;
                 } else if coup2.is_none() {
                     return a;
                 }
                 if val1 > val2 { a } else { b }
             })
             .unwrap_or((None, -state.value()));
        (coup, val)
    }
}


fn alphabeta_sorted(profondeur: u8, mut alpha: i8, beta: i8, state: Configuration) -> (Option<Movement>, i8) {
    if profondeur == 0 || state.movements().peekable().peek().is_none() {
        (None, -state.value())
    } else {
        let mut best_move = None;
        let mut best_val = -127;
        
        if profondeur >= 4 {
            let mut mouvements_ordonnes = state.movements().collect::<Vec<Movement>>();
            mouvements_ordonnes.sort_by_key(|&coup| -alphabeta(2, alpha, beta, state.play(&coup)).1);
            for coup in mouvements_ordonnes {
                let (_, val0) = alphabeta(profondeur - 1, -beta, -alpha, state.play(&coup));
                let val = -val0;
                if val > best_val {
                    best_val = val;
                    best_move = Some(coup);
                    if best_val > alpha {
                        alpha = best_val;
                        if alpha >= beta { // or val ?
                            break; // Problème pr paralléliser
                        }
                    }
                }
            }
            (best_move, best_val)
        } else {
            for coup in state.movements() {
                let (_, val0) = alphabeta(profondeur - 1, -beta, -alpha, state.play(&coup));
                let val = -val0;
                if val > best_val {
                    best_val = val;
                    best_move = Some(coup);
                    if best_val > alpha {
                        alpha = best_val;
                        if alpha >= beta { // or val ?
                            break; // Problème pr paralléliser
                        }
                    }
                }
            }
            (best_move, best_val)
        }
    }
}
