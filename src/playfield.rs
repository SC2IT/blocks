use ggez::glam::IVec2;

use crate::rustomino::{translated, Rotation, Rustomino, RustominoState, RustominoType};
use std::fmt::Display;

pub const PLAYFIELD_SLOTS: [usize; 2] = [10, 22];
pub const LINE_CLEAR_FADE: f64 = 0.5; // fraction of the line clear each block spends fading
pub const PLAYFIELD_SIZE: [i32; 2] = [10, 20];

type PlayfieldSlots = [[SlotState; PLAYFIELD_SLOTS[0]]; PLAYFIELD_SLOTS[1]];

#[derive(Debug)]
pub struct LineClear {
    pub lines: Vec<usize>,
    pub origins: Vec<usize>, // columns where the locked rustomino landed
    pub time: f64,           // seconds since the lines were completed
    pub duration: f64,       // seconds until the lines are removed
}

impl LineClear {
    /// fraction of the line clear completed, 0.0 to 1.0
    pub fn progress(&self) -> f64 {
        (self.time / self.duration).clamp(0.0, 1.0)
    }

    /// columns between x and the nearest origin
    pub fn distance(&self, x: usize) -> usize {
        self.origins
            .iter()
            .map(|origin| origin.abs_diff(x))
            .min()
            .unwrap_or(0)
    }

    /// nearest origin to x
    pub fn origin(&self, x: usize) -> usize {
        self.origins
            .iter()
            .copied()
            .min_by_key(|origin| origin.abs_diff(x))
            .unwrap_or(x)
    }

    /// fraction of the line clear when block x starts fading
    pub fn wipe_start(&self, x: usize) -> f64 {
        let max_distance = self.max_distance().max(1) as f64;
        self.distance(x) as f64 / max_distance * (1.0 - LINE_CLEAR_FADE)
    }

    pub fn max_distance(&self) -> usize {
        (0..PLAYFIELD_SLOTS[0])
            .map(|x| self.distance(x))
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct Playfield {
    pub slots: PlayfieldSlots, // locked blocks only
    pub active_rustomino: Option<Rustomino>,
    pub ghost_rustomino: Option<Rustomino>,
    pub line_clear: Option<LineClear>,
}

impl Playfield {
    pub fn new() -> Self {
        log::info!("Initializing Playfield");
        Playfield {
            slots: [[SlotState::Empty; PLAYFIELD_SLOTS[0]]; PLAYFIELD_SLOTS[1]],
            active_rustomino: None,
            ghost_rustomino: None,
            line_clear: None,
        }
    }

    /// Adds a new rustomino to the playfield
    /// returns false if there was a collision
    /// while adding the block (game over)
    pub fn set_active(&mut self, rustomino: Rustomino) -> bool {
        log::info!("playing new rustomino: {:?}", rustomino.rtype);
        log::trace!("new rustomino: {:?}", rustomino);
        let ok = !check_collision(&self.slots, rustomino.playfield_slots());
        self.active_rustomino = Some(rustomino);
        self.update_ghost_rustomino();
        ok
    }

    pub fn take_active(&mut self) -> Option<Rustomino> {
        let active_rustomino = self.active_rustomino.take()?;

        log::debug!("taking active rustomino: {:?}", active_rustomino.rtype);
        log::trace!("rustomino: {:?}", active_rustomino);
        self.update_ghost_rustomino();
        Some(active_rustomino.reset())
    }

    /// checks to see if the playfield needs the next rustomino
    pub fn ready_for_next(&self) -> bool {
        self.active_rustomino.is_none()
    }

    // checking if rustomino can fall
    pub fn active_can_fall(&self) -> bool {
        self.active_rustomino.as_ref().is_some_and(|rustomino| {
            !check_collision(
                &self.slots,
                rustomino.translated(&TranslationDirection::DOWN_TRANSLATION),
            )
        })
    }

    /// lowest row occupied by the active rustomino
    pub fn active_lowest_row(&self) -> Option<i32> {
        self.active_rustomino
            .as_ref()
            .and_then(|r| r.playfield_slots().iter().map(|s| s.y).min())
    }

    pub fn get_active_state(&self) -> Option<RustominoState> {
        self.active_rustomino
            .as_ref()
            .map(|active_rustomino| active_rustomino.state)
    }

    pub fn set_active_state(&mut self, new_state: RustominoState) {
        if let Some(active_rustomino) = self.active_rustomino.as_mut() {
            active_rustomino.set_state(new_state)
        }
    }

    /// Attempt to rotate the active rustomino
    pub fn rotate_active(&mut self, rotation: Rotation) -> bool {
        let Some(active_rustomino) = self.active_rustomino.as_mut() else {
            return false;
        };

        // check to see if the block can be rotated with or without a wall kick
        let Some(wall_kick_trans) = check_rotation(&self.slots, active_rustomino, &rotation) else {
            return false;
        };

        active_rustomino.rotate(&rotation, &wall_kick_trans);
        self.update_ghost_rustomino();
        true
    }

    /// Attempt to translate the active rustomino.
    /// Return true if possible
    pub fn translate_active(&mut self, direction: TranslationDirection) -> bool {
        let Some(active_rustomino) = self.active_rustomino.as_mut() else {
            return false;
        };

        // check to see if the translation would cause a collision with a locked block
        let translation = direction.get_translation();
        if check_collision(&self.slots, active_rustomino.translated(&translation)) {
            log::debug!("cannot translate, collision detected");
            return false;
        }

        active_rustomino.translate(translation);
        self.update_ghost_rustomino();
        true
    }

    /// returns the slots it dropped from
    pub fn hard_drop_active(&mut self) -> Option<[IVec2; 4]> {
        let active_rustomino = self.active_rustomino.as_mut()?;
        let from = active_rustomino.playfield_slots();
        let delta = get_hard_drop_translation(&self.slots, active_rustomino);
        active_rustomino.translate(delta);
        Some(from)
    }

    /// lock the active rustomino
    /// returns the slots it was locked into
    pub fn lock_active(&mut self) -> Option<[IVec2; 4]> {
        let active_rustomino = self.active_rustomino.take()?;
        log::info!("locking rustomino: {:?}", active_rustomino.rtype);
        log::trace!("rustomino: {:?}", active_rustomino);

        let slots = active_rustomino.playfield_slots();
        let wear = if active_rustomino.tough {
            Wear::Tough
        } else {
            Wear::Whole
        };
        for slot in slots {
            self.slots[slot.y as usize][slot.x as usize] =
                SlotState::Locked(active_rustomino.rtype, wear);
        }
        self.update_ghost_rustomino();
        Some(slots)
    }

    /// apply gravity to the active rustomino
    pub fn apply_gravity(&mut self) {
        log::trace!("applying gravity");
        if let Some(active_rustomino) = self.active_rustomino.as_mut() {
            active_rustomino.translate(TranslationDirection::DOWN_TRANSLATION);
        }
    }

    /// start clearing any completed lines
    /// returns the number of lines being cleared
    pub fn start_line_clear(&mut self, duration: f64, landed: &[IVec2]) -> usize {
        let lines = self.get_complete_lines();
        if lines.is_empty() {
            return 0;
        }
        log::info!("clearing completed lines: {:?}", lines);
        let num_lines = lines.len();
        let mut origins: Vec<usize> = landed.iter().map(|slot| slot.x as usize).collect();
        origins.sort_unstable();
        origins.dedup();
        self.line_clear = Some(LineClear {
            lines,
            origins,
            time: 0.0,
            duration,
        });
        num_lines
    }

    pub fn is_clearing_lines(&self) -> bool {
        self.line_clear.is_some()
    }

    /// advance the line clear, removing the lines when it finishes
    pub fn update_line_clear(&mut self, delta_time: f64) {
        let Some(line_clear) = self.line_clear.as_mut() else {
            return;
        };
        line_clear.time += delta_time;
        if line_clear.time >= line_clear.duration {
            let line_clear = self.line_clear.take().expect("line clear in progress");
            self.remove_lines(&line_clear.lines);
        }
    }

    fn remove_lines(&mut self, completed_lines: &[usize]) {
        log::trace!("clearing lines before: playfield:\n{}", self);

        // column by column: tough blocks in a completed line survive, cracked, everything
        // else in it goes, and what's above drops down onto what's left
        for x in 0..PLAYFIELD_SLOTS[0] {
            let mut write = 0;
            for read in 0..self.slots.len() {
                let slot = self.slots[read][x];
                let kept = if completed_lines.contains(&read) {
                    match slot {
                        SlotState::Locked(rtype, Wear::Tough) => {
                            Some(SlotState::Locked(rtype, Wear::Cracked))
                        }
                        _ => None,
                    }
                } else {
                    Some(slot)
                };
                if let Some(slot) = kept {
                    self.slots[write][x] = slot;
                    write += 1;
                }
            }
            for line in &mut self.slots[write..] {
                line[x] = SlotState::Empty;
            }
        }

        log::trace!("clearing lines after: playfield:\n{}", self);
    }

    /// Returns the get complete lines of this [`Playfield`].
    fn get_complete_lines(&self) -> Vec<usize> {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, line)| {
                line.iter()
                    .all(|slot| matches!(slot, SlotState::Locked(..)))
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn update_ghost_rustomino(&mut self) {
        self.ghost_rustomino = self.active_rustomino.as_ref().map(|active_rustomino| {
            let mut ghost_rustomino = active_rustomino.clone();
            ghost_rustomino.translate(get_hard_drop_translation(&self.slots, active_rustomino));
            ghost_rustomino
        });
    }
}

fn get_hard_drop_translation(playfield_slots: &PlayfieldSlots, rustomino: &Rustomino) -> IVec2 {
    let mut translation = IVec2::ZERO;
    while !check_collision(
        playfield_slots,
        rustomino.translated(&(translation + TranslationDirection::DOWN_TRANSLATION)),
    ) {
        translation += TranslationDirection::DOWN_TRANSLATION;
    }
    log::debug!("hard_drop_translation: {:?}", translation);
    translation
}

/// check to see if the provided block locations collide with other locked blocks
/// or with walls
fn check_collision(playfield_slots: &PlayfieldSlots, block_locations: [IVec2; 4]) -> bool {
    for location in block_locations {
        // check for left and right wall collisions
        if location[0] < 0 || location[0] >= PLAYFIELD_SLOTS[0] as i32 {
            log::trace!("collided with left/right wall: {:?}", block_locations);
            return true;
        }
        if location[1] >= PLAYFIELD_SLOTS[1] as i32 {
            log::trace!("collided with top wall: {:?}", block_locations);
            return true;
        }
        // check for bottom wall collision
        if location[1] < 0 {
            log::trace!("collided with bottom wall: {:?}", block_locations);
            return true;
        }
        // slots[y][x]
        if matches!(
            playfield_slots[location[1] as usize][location[0] as usize],
            SlotState::Locked(..)
        ) {
            log::trace!("collided with locked block: {:?}", block_locations);
            return true;
        }
    }
    false
}

fn check_rotation(
    playfield_slots: &PlayfieldSlots,
    rustomino: &Rustomino,
    rotation: &Rotation,
) -> Option<IVec2> {
    let wall_kick_tests = rustomino.wall_kick_tests(rotation);
    let rotated_blocks = rustomino.rotated(rotation);
    wall_kick_tests
        .iter()
        .find(|x| !check_collision(playfield_slots, translated(&rotated_blocks, x)))
        .copied()
}

// display the playfield's slot states for debugging
impl Display for Playfield {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let active = self.active_rustomino.as_ref().map(|r| r.playfield_slots());
        let ghost = self.ghost_rustomino.as_ref().map(|r| r.playfield_slots());
        let contains = |blocks: &Option<[IVec2; 4]>, slot: IVec2| {
            blocks.is_some_and(|blocks| blocks.contains(&slot))
        };
        for (y, row) in self.slots.iter().enumerate().rev() {
            if y + 1 == PLAYFIELD_SIZE[1] as usize {
                writeln!(f, "{}", "-".repeat(PLAYFIELD_SLOTS[0] * 2))?;
            }
            for (x, slot) in row.iter().enumerate() {
                let pos = IVec2::new(x as i32, y as i32);
                if contains(&active, pos) {
                    write!(f, " #")?;
                } else if contains(&ghost, pos) {
                    write!(f, " %")?;
                } else {
                    write!(f, "{slot}")?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "{}", "-".repeat(PLAYFIELD_SLOTS[0] * 2))?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum TranslationDirection {
    Left,
    Right,
    Down,
}

impl TranslationDirection {
    const LEFT_TRANSLATION: IVec2 = IVec2::new(-1, 0);
    const RIGHT_TRANSLATION: IVec2 = IVec2::new(1, 0);
    const DOWN_TRANSLATION: IVec2 = IVec2::new(0, -1);
    pub fn get_translation(&self) -> IVec2 {
        match self {
            TranslationDirection::Left => Self::LEFT_TRANSLATION,
            TranslationDirection::Right => Self::RIGHT_TRANSLATION,
            TranslationDirection::Down => Self::DOWN_TRANSLATION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotState {
    Empty,
    Locked(RustominoType, Wear),
}

/// how much a locked block has been through
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wear {
    Whole,   // an ordinary block
    Tough,   // from a tough piece, survives its first line clear
    Cracked, // a tough block after its first clear, the next one takes it
}

impl Display for SlotState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SlotState::Empty => write!(f, "  ")?,
            SlotState::Locked(_, Wear::Whole) => write!(f, " @")?,
            SlotState::Locked(_, Wear::Tough) => write!(f, " #")?,
            SlotState::Locked(_, Wear::Cracked) => write!(f, " %")?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: usize = PLAYFIELD_SLOTS[0];

    fn whole() -> SlotState {
        SlotState::Locked(RustominoType::T, Wear::Whole)
    }

    fn tough() -> SlotState {
        SlotState::Locked(RustominoType::T, Wear::Tough)
    }

    fn cracked() -> SlotState {
        SlotState::Locked(RustominoType::T, Wear::Cracked)
    }

    #[test]
    fn a_plain_line_clears_and_the_rest_drops() {
        let mut playfield = Playfield::new();
        playfield.slots[0] = [whole(); W];
        playfield.slots[1][4] = whole();
        playfield.remove_lines(&[0]);
        assert_eq!(playfield.slots[0][4], whole());
        assert_eq!(playfield.slots[0][3], SlotState::Empty);
        assert_eq!(playfield.slots[1][4], SlotState::Empty);
    }

    #[test]
    fn tough_blocks_survive_cracked_and_the_column_drops_onto_them() {
        let mut playfield = Playfield::new();
        playfield.slots[0] = [whole(); W];
        playfield.slots[0][2] = tough();
        playfield.slots[1][2] = whole(); // above the tough block
        playfield.slots[1][5] = whole(); // above an ordinary one
        playfield.remove_lines(&[0]);
        // the tough block stays, cracked, and its column lands on it
        assert_eq!(playfield.slots[0][2], cracked());
        assert_eq!(playfield.slots[1][2], whole());
        // elsewhere the line goes and what's above drops into it
        assert_eq!(playfield.slots[0][5], whole());
        assert_eq!(playfield.slots[1][5], SlotState::Empty);
        assert_eq!(playfield.slots[0][0], SlotState::Empty);
    }

    #[test]
    fn cracked_blocks_go_on_the_next_clear() {
        let mut playfield = Playfield::new();
        playfield.slots[0] = [whole(); W];
        playfield.slots[0][7] = cracked();
        playfield.remove_lines(&[0]);
        assert!(playfield.slots[0]
            .iter()
            .all(|slot| *slot == SlotState::Empty));
    }

    #[test]
    fn holes_below_the_cleared_line_stay() {
        // columns drop only into the cleared line, not into holes further down
        let mut playfield = Playfield::new();
        playfield.slots[1] = [whole(); W];
        playfield.slots[2][3] = whole();
        playfield.remove_lines(&[1]);
        assert_eq!(playfield.slots[0][3], SlotState::Empty);
        assert_eq!(playfield.slots[1][3], whole());
    }

    #[test]
    fn a_tough_piece_locks_as_tough_blocks() {
        let mut playfield = Playfield::new();
        let mut piece = crate::rustomino::Rustomino::new(RustominoType::O);
        piece.tough = true;
        playfield.set_active(piece);
        let slots = playfield.lock_active().expect("a piece to lock");
        for slot in slots {
            assert_eq!(
                playfield.slots[slot.y as usize][slot.x as usize],
                SlotState::Locked(RustominoType::O, Wear::Tough)
            );
        }
    }
}
