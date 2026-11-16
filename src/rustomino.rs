use ::rand::seq::{IndexedRandom, SliceRandom};
use ::rand::RngExt;
use ggez::{glam::IVec2, graphics::Color};
use strum::{EnumIter, IntoEnumIterator};

use crate::game::Mode;

const I_START_TRANSLATION: IVec2 = IVec2::new(3, 18);
const O_T_L_J_S_Z_START_TRANSLATION: IVec2 = IVec2::new(3, 19);

const I_BLOCKS: [IVec2; 4] = [
    IVec2::new(0, 2),
    IVec2::new(1, 2),
    IVec2::new(2, 2),
    IVec2::new(3, 2),
];
const O_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 2),
    IVec2::new(2, 2),
    IVec2::new(2, 1),
    IVec2::new(1, 1),
];
const T_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 1),
    IVec2::new(0, 1),
    IVec2::new(1, 2),
    IVec2::new(2, 1),
];
const L_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 1),
    IVec2::new(0, 1),
    IVec2::new(2, 2),
    IVec2::new(2, 1),
];
const J_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 1),
    IVec2::new(0, 1),
    IVec2::new(0, 2),
    IVec2::new(2, 1),
];
const S_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 1),
    IVec2::new(0, 1),
    IVec2::new(1, 2),
    IVec2::new(2, 2),
];
const Z_BLOCKS: [IVec2; 4] = [
    IVec2::new(1, 1),
    IVec2::new(0, 2),
    IVec2::new(1, 2),
    IVec2::new(2, 1),
];

const I_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(2, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, -2),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(1, -2),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(-2, 1),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(-2, -1),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(1, 2),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(-1, 2),
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(2, -1),
    ],
];

const O_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(1, 0),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(-1, 0),
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 1),
        IVec2::new(1, 0),
        IVec2::new(0, -1),
        IVec2::new(-1, 0),
    ],
];

const T_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(0, 0),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
        IVec2::new(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, 0),
        IVec2::new(1, -1),
        IVec2::new(-1, -1),
        IVec2::new(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(0, 0),
        IVec2::new(-1, -1),
        IVec2::new(-1, 1),
        IVec2::new(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 0),
        IVec2::new(-1, 1),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
    ],
];

const L_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(0, 0),
        IVec2::new(1, 1),
        IVec2::new(0, -2),
        IVec2::new(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, 0),
        IVec2::new(1, -1),
        IVec2::new(-2, 0),
        IVec2::new(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(0, 0),
        IVec2::new(-1, -1),
        IVec2::new(0, 2),
        IVec2::new(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 0),
        IVec2::new(-1, 1),
        IVec2::new(2, 0),
        IVec2::new(1, -1),
    ],
];

const J_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(0, 0),
        IVec2::new(1, 1),
        IVec2::new(2, 0),
        IVec2::new(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, 0),
        IVec2::new(1, -1),
        IVec2::new(0, -2),
        IVec2::new(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(0, 0),
        IVec2::new(-1, -1),
        IVec2::new(-2, 0),
        IVec2::new(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 0),
        IVec2::new(-1, 1),
        IVec2::new(0, 2),
        IVec2::new(1, -1),
    ],
];

const S_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(0, 0),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
        IVec2::new(0, -2),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, 0),
        IVec2::new(1, -1),
        IVec2::new(-1, -1),
        IVec2::new(-2, 0),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(0, 0),
        IVec2::new(-1, -1),
        IVec2::new(-1, 1),
        IVec2::new(0, 2),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 0),
        IVec2::new(-1, 1),
        IVec2::new(1, 1),
        IVec2::new(2, 0),
    ],
];

const Z_ROTATIONS: [[IVec2; 4]; 4] = [
    [
        // N>>E || -(E>>N)
        IVec2::new(0, 0),
        IVec2::new(2, 0),
        IVec2::new(1, -1),
        IVec2::new(-1, -1),
    ],
    [
        // E>>S || -(S>>E)
        IVec2::new(0, 0),
        IVec2::new(0, -2),
        IVec2::new(-1, -1),
        IVec2::new(-1, 1),
    ],
    [
        // S>>W || -(W>>S)
        IVec2::new(0, 0),
        IVec2::new(-2, 0),
        IVec2::new(-1, 1),
        IVec2::new(1, 1),
    ],
    [
        // W>>N || -(N>>W)
        IVec2::new(0, 0),
        IVec2::new(0, 2),
        IVec2::new(1, 1),
        IVec2::new(1, -1),
    ],
];

const JLSTZ_WALL_KICK_TESTS: [[IVec2; 5]; 8] = [
    [
        // N->E (0, 0),(-1, 0),(-1,1),( 0,-2),(-1,-2)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(-1, 1),
        IVec2::new(0, -2),
        IVec2::new(-1, -2),
    ],
    [
        // E->N (0, 0),(1, 0),(1,-1),( 0,2),(1,2)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(1, -1),
        IVec2::new(0, 2),
        IVec2::new(1, 2),
    ],
    [
        // E->S (0, 0),(1, 0),(1,-1),(0, 2),(1, 2)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(1, -1),
        IVec2::new(0, 2),
        IVec2::new(1, 2),
    ],
    [
        // S->E ( 0, 0),(-1, 0),(-1,1),( 0,-2),(-1,-2)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(-1, 1),
        IVec2::new(0, -2),
        IVec2::new(-1, -2),
    ],
    [
        // E->W ( 0, 0),(1, 0),(1,1),( 0,-2),(1,-2)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(1, 1),
        IVec2::new(0, -2),
        IVec2::new(1, -2),
    ],
    [
        // W->E ( 0, 0),(-1, 0),(-1,-1),( 0,2),(-1,2)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(-1, -1),
        IVec2::new(0, 2),
        IVec2::new(-1, 2),
    ],
    [
        // W->N ( 0, 0),(-1, 0),(-1,-1),( 0,2),(-1,2)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(-1, -1),
        IVec2::new(0, 2),
        IVec2::new(-1, 2),
    ],
    [
        // N->W (0, 0),(1, 0),(1, 1),(0, -2),(1, -2)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(1, 1),
        IVec2::new(0, -2),
        IVec2::new(1, -2),
    ],
];

const I_WALL_KICK_TESTS: [[IVec2; 5]; 8] = [
    [
        // N->E ( 0, 0),(-2, 0),(1, 0),(-2,-1),(1,2)
        IVec2::new(0, 0),
        IVec2::new(-2, 0),
        IVec2::new(1, 0),
        IVec2::new(-2, -1),
        IVec2::new(1, 2),
    ],
    [
        // E->N ( 0, 0),(2, 0),(-1, 0),(2,1),(-1,-2)
        IVec2::new(0, 0),
        IVec2::new(2, 0),
        IVec2::new(-1, 0),
        IVec2::new(2, 1),
        IVec2::new(-1, -2),
    ],
    [
        // E->S ( 0, 0),(-1, 0),(2, 0),(-1,2),(2,-1)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(2, 0),
        IVec2::new(-1, 2),
        IVec2::new(2, -1),
    ],
    [
        // S->E ( 0, 0),(1, 0),(-2, 0),(1,-2),(-2,1)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(-2, 0),
        IVec2::new(1, -2),
        IVec2::new(-2, 1),
    ],
    [
        // E->W ( 0, 0),(2, 0),(-1, 0),(2,1),(-1,-2)
        IVec2::new(0, 0),
        IVec2::new(2, 0),
        IVec2::new(-1, 0),
        IVec2::new(2, 1),
        IVec2::new(-1, -2),
    ],
    [
        // W->E ( 0, 0),(-2, 0),(1, 0),(-2,-1),(1,2)
        IVec2::new(0, 0),
        IVec2::new(-2, 0),
        IVec2::new(1, 0),
        IVec2::new(-2, -1),
        IVec2::new(1, 2),
    ],
    [
        // W->N ( 0, 0),(1, 0),(-2, 0),(1,-2),(-2,1)
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(-2, 0),
        IVec2::new(1, -2),
        IVec2::new(-2, 1),
    ],
    [
        // N->W ( 0, 0),(-1, 0),(2, 0),(-1,2),(2,-1)
        IVec2::new(0, 0),
        IVec2::new(-1, 0),
        IVec2::new(2, 0),
        IVec2::new(-1, 2),
        IVec2::new(2, -1),
    ],
];

// glitch pieces: odd shapes that turn up now and then, more often at higher levels
const GLITCH_CHANCE: f64 = 0.02; // chance per piece at level 0
const GLITCH_CHANCE_PER_LEVEL: f64 = 0.004;
const GLITCH_CHANCE_MAX: f64 = 0.08;

/// shapes a glitch piece can take, in a 3x3 box like J L S T Z
/// or a 4x4 box like I, four blocks like any piece, but joined at
/// corners or broken by gaps
#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq)]
pub enum GlitchShape {
    Split,   // two columns with a gap between
    Diamond, // around a hollow center, joined at the corners
    Corners, // the four corners of the box
    Antlers, // a Y, two prongs on a stem
    Kink,    // a short wiggle with a block off one corner
    Hook,    // two pairs, offset by a gap
    Drip,    // an I with its end dropped a row, 4x4 box
}

impl GlitchShape {
    /// shapes that can turn up in a mode: standard gets the gentle ones,
    /// hard adds the hollow and corner-joined ones
    fn in_mode(mode: Mode) -> Vec<GlitchShape> {
        GlitchShape::iter()
            .filter(|shape| mode == Mode::Hard || shape.gentle())
            .collect()
    }

    /// mostly joined edge to edge, with one gap
    fn gentle(self) -> bool {
        matches!(
            self,
            GlitchShape::Split | GlitchShape::Kink | GlitchShape::Hook | GlitchShape::Drip
        )
    }

    /// width of the box it turns in, 3 like J L S T Z or 4 like I
    fn box_size(self) -> i32 {
        match self {
            GlitchShape::Drip => 4,
            _ => 3,
        }
    }

    fn blocks(self) -> Vec<IVec2> {
        // y is up, row 2 is the top of the box
        let cells: &[(i32, i32)] = match self {
            GlitchShape::Split => &[(0, 1), (0, 2), (2, 1), (2, 2)],
            GlitchShape::Diamond => &[(1, 2), (0, 1), (2, 1), (1, 0)],
            GlitchShape::Corners => &[(0, 2), (2, 2), (0, 0), (2, 0)],
            GlitchShape::Antlers => &[(0, 2), (2, 2), (1, 1), (1, 0)],
            GlitchShape::Kink => &[(0, 2), (1, 1), (2, 1), (1, 0)],
            GlitchShape::Hook => &[(0, 2), (1, 2), (1, 0), (2, 0)],
            // in the I's row of its 4x4 box
            GlitchShape::Drip => &[(0, 2), (1, 2), (2, 2), (3, 1)],
        };
        cells.iter().map(|&(x, y)| IVec2::new(x, y)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Rustomino {
    pub rtype: RustominoType,
    pub state: RustominoState,
    pub rotation: RustominoRotation,
    pub blocks: Vec<IVec2>,
    pub translation: IVec2,
    pub glitch: Option<GlitchShape>,
}

impl Rustomino {
    pub fn new(rtype: RustominoType) -> Rustomino {
        if rtype == RustominoType::Glitch {
            return Rustomino::glitch(GlitchShape::Split);
        }
        let (rotation, blocks, translation) = {
            match rtype {
                RustominoType::I => (
                    RustominoRotation::new(&I_ROTATIONS),
                    I_BLOCKS,
                    I_START_TRANSLATION,
                ),
                RustominoType::O => (
                    RustominoRotation::new(&O_ROTATIONS),
                    O_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::T => (
                    RustominoRotation::new(&T_ROTATIONS),
                    T_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::L => (
                    RustominoRotation::new(&L_ROTATIONS),
                    L_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::J => (
                    RustominoRotation::new(&J_ROTATIONS),
                    J_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::S => (
                    RustominoRotation::new(&S_ROTATIONS),
                    S_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::Z => (
                    RustominoRotation::new(&Z_ROTATIONS),
                    Z_BLOCKS,
                    O_T_L_J_S_Z_START_TRANSLATION,
                ),
                RustominoType::Glitch => unreachable!("glitch pieces are built by glitch()"),
            }
        };
        Rustomino {
            rtype,
            state: RustominoState::Falling { time: 0. },
            rotation,
            blocks: blocks.to_vec(),
            translation,
            glitch: None,
        }
    }

    /// an odd shaped glitch piece, it turns about the center of its box
    pub fn glitch(shape: GlitchShape) -> Rustomino {
        Rustomino {
            rtype: RustominoType::Glitch,
            state: RustominoState::Falling { time: 0. },
            rotation: RustominoRotation::pivot(),
            blocks: shape.blocks(),
            translation: match shape.box_size() {
                4 => I_START_TRANSLATION,
                _ => O_T_L_J_S_Z_START_TRANSLATION,
            },
            glitch: Some(shape),
        }
    }

    pub fn reset(self) -> Rustomino {
        match self.glitch {
            Some(shape) => Rustomino::glitch(shape),
            None => Rustomino::new(self.rtype),
        }
    }

    pub fn translate(&mut self, delta: IVec2) {
        self.translation += delta;
    }

    pub fn translated(&self, delta: &IVec2) -> Vec<IVec2> {
        translated(&translated(&self.blocks, &self.translation), delta)
    }

    pub fn playfield_slots(&self) -> Vec<IVec2> {
        self.translated(&IVec2::ZERO)
    }

    /// the blocks in the piece's own box after turning
    fn turned_blocks(&self, rotation: &Rotation) -> Vec<IVec2> {
        match self.rotation.get_rotation_trans(rotation) {
            Some(deltas) => self
                .blocks
                .iter()
                .zip(deltas)
                .map(|(b, d)| *b + d)
                .collect(),
            // a quarter turn about the center of the box
            None => {
                let edge = self.glitch.map_or(3, GlitchShape::box_size) - 1;
                self.blocks
                    .iter()
                    .map(|b| match rotation {
                        Rotation::Cw => IVec2::new(b.y, edge - b.x),
                        Rotation::Ccw => IVec2::new(edge - b.y, b.x),
                    })
                    .collect()
            }
        }
    }

    pub fn rotate(&mut self, rotation: &Rotation, translation: &IVec2) {
        self.blocks = self.turned_blocks(rotation);
        // apply the wall kick to the position, not the shape
        self.translation += *translation;

        self.rotation.rotate(rotation);
    }

    pub fn rotated(&self, rotation: &Rotation) -> Vec<IVec2> {
        translated(&self.turned_blocks(rotation), &self.translation)
    }

    pub fn wall_kick_tests(&self, rotation: &Rotation) -> [IVec2; 5] {
        // glitch pieces in a 4x4 box kick like the I
        let rtype = match self.glitch.map(GlitchShape::box_size) {
            Some(4) => RustominoType::I,
            _ => self.rtype,
        };
        self.rotation.get_wall_kick_tests(rtype, rotation)
    }

    pub fn set_state(&mut self, state: RustominoState) {
        log::trace!("setting rustomino state: {:?}", state);
        self.state = state;
    }
}

pub fn translated(blocks: &[IVec2], delta: &IVec2) -> Vec<IVec2> {
    blocks.iter().map(|b| *b + *delta).collect()
}

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq)]
pub enum RustominoType {
    I,
    O,
    T,
    L,
    J,
    S,
    Z,
    // left out of the bag, see RustominoBag::get_next
    #[strum(disabled)]
    Glitch,
}

impl RustominoType {
    // piece colors in I O T L J S Z order
    const PALETTE: [Color; 7] = [
        Color::new(1.0, 0.45, 0.38, 1.0),  // coral
        Color::new(0.95, 0.35, 0.72, 1.0), // rose
        Color::new(1.0, 0.8, 0.25, 1.0),   // gold
        Color::new(0.62, 0.48, 1.0, 1.0),  // violet
        Color::new(0.9, 0.84, 0.7, 1.0),   // sand
        Color::new(0.32, 0.64, 1.0, 1.0),  // sky
        Color::new(0.15, 0.85, 0.55, 1.0), // emerald
    ];

    // pale cyan, the glitch look splits it into magenta and cyan
    const GLITCH_COLOR: Color = Color::new(0.78, 1.0, 0.96, 1.0);

    pub fn color(&self) -> Color {
        match self {
            RustominoType::Glitch => RustominoType::GLITCH_COLOR,
            _ => RustominoType::PALETTE[*self as usize],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RustominoState {
    Falling { time: f64 },
    Lockdown { time: f64 },
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    N,
    E,
    S,
    W,
}

impl Direction {
    fn rotate(&self, rotation: &Rotation) -> Direction {
        match self {
            Direction::N => match rotation {
                Rotation::Cw => Direction::E,
                Rotation::Ccw => Direction::W,
            },
            Direction::E => match rotation {
                Rotation::Cw => Direction::S,
                Rotation::Ccw => Direction::N,
            },
            Direction::S => match rotation {
                Rotation::Cw => Direction::W,
                Rotation::Ccw => Direction::E,
            },
            Direction::W => match rotation {
                Rotation::Cw => Direction::N,
                Rotation::Ccw => Direction::S,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Rotation {
    Cw,
    Ccw,
}

#[derive(Debug, Clone, Copy)]
pub struct RustominoRotation {
    direction: Direction,
    // N>>E, E>>S, S>>W, W>>N
    // None turns about the center of a 3x3 box instead
    translations: Option<&'static [[IVec2; 4]; 4]>,
}

impl RustominoRotation {
    fn new(translations: &'static [[IVec2; 4]; 4]) -> Self {
        Self {
            direction: Direction::N,
            translations: Some(translations),
        }
    }

    fn pivot() -> Self {
        Self {
            direction: Direction::N,
            translations: None,
        }
    }

    fn get_rotation_trans(&self, rotation: &Rotation) -> Option<[IVec2; 4]> {
        let [n2e, e2s, s2w, w2n] = self.translations?;
        Some(match self.direction {
            Direction::N => match rotation {
                Rotation::Cw => *n2e,
                Rotation::Ccw => neg_trans(*w2n),
            },
            Direction::E => match rotation {
                Rotation::Cw => *e2s,
                Rotation::Ccw => neg_trans(*n2e),
            },
            Direction::S => match rotation {
                Rotation::Cw => *s2w,
                Rotation::Ccw => neg_trans(*e2s),
            },
            Direction::W => match rotation {
                Rotation::Cw => *w2n,
                Rotation::Ccw => neg_trans(*s2w),
            },
        })
    }

    fn get_wall_kick_tests(&self, rtype: RustominoType, rotation: &Rotation) -> [IVec2; 5] {
        match self.direction {
            Direction::N => match rotation {
                Rotation::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[0],
                    _ => JLSTZ_WALL_KICK_TESTS[0],
                },
                Rotation::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[7],
                    _ => JLSTZ_WALL_KICK_TESTS[7],
                },
            },
            Direction::E => match rotation {
                Rotation::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[2],
                    _ => JLSTZ_WALL_KICK_TESTS[2],
                },
                Rotation::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[1],
                    _ => JLSTZ_WALL_KICK_TESTS[1],
                },
            },
            Direction::S => match rotation {
                Rotation::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[4],
                    _ => JLSTZ_WALL_KICK_TESTS[4],
                },
                Rotation::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[3],
                    _ => JLSTZ_WALL_KICK_TESTS[3],
                },
            },
            Direction::W => match rotation {
                Rotation::Cw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[6],
                    _ => JLSTZ_WALL_KICK_TESTS[6],
                },
                Rotation::Ccw => match rtype {
                    RustominoType::I => I_WALL_KICK_TESTS[5],
                    _ => JLSTZ_WALL_KICK_TESTS[5],
                },
            },
        }
    }

    fn rotate(&mut self, rotation: &Rotation) {
        self.direction = self.direction.rotate(rotation)
    }
}

#[inline(always)]
fn neg_trans(block_trans: [IVec2; 4]) -> [IVec2; 4] {
    [
        -block_trans[0],
        -block_trans[1],
        -block_trans[2],
        -block_trans[3],
    ]
}

pub struct RustominoBag {
    bag: Vec<RustominoType>, // contains the next rustomino types, shuffled
    rng: rand_xoshiro::Xoshiro256PlusPlus,
    last_glitched: bool, // never two glitch pieces in a row
}

impl RustominoBag {
    pub fn new() -> Self {
        RustominoBag {
            bag: Vec::new(),
            rng: rand::make_rng(),
            last_glitched: false,
        }
    }

    pub fn get_next(&mut self, level: usize, mode: Mode) -> Rustomino {
        // now and then slip in a glitch piece, the bag keeps its piece for next time
        let glitch = !self.last_glitched && self.rng.random_bool(glitch_chance(level));
        self.last_glitched = glitch;
        if glitch {
            let shape = *GlitchShape::in_mode(mode)
                .choose(&mut self.rng)
                .expect("no glitch shapes");
            log::info!("next rustomino is a glitch: {:?}", shape);
            return Rustomino::glitch(shape);
        }

        // make sure the bag isn't empty
        self.fill_bag();

        let rtype = self.bag.pop().expect("rustomino bag is empty");
        log::info!("next rustomino type: {:?}", rtype);

        Rustomino::new(rtype)
    }

    // add one of each rustomino type to bag
    // then shuffle the bag
    fn fill_bag(&mut self) {
        if !self.bag.is_empty() {
            return;
        }
        self.bag.append(&mut RustominoType::iter().collect());
        self.bag.shuffle(&mut self.rng);
        log::debug!("filled rustomino bag: {:?}", self.bag);
    }
}

/// chance the next piece is a glitch piece at a level
fn glitch_chance(level: usize) -> f64 {
    (GLITCH_CHANCE + level as f64 * GLITCH_CHANCE_PER_LEVEL).min(GLITCH_CHANCE_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut blocks: Vec<IVec2>) -> Vec<IVec2> {
        blocks.sort_by_key(|b| (b.x, b.y));
        blocks
    }

    #[test]
    fn glitch_pieces_turn_back_to_their_start() {
        for shape in GlitchShape::iter() {
            let start = Rustomino::glitch(shape);
            for rotation in [Rotation::Cw, Rotation::Ccw] {
                let mut piece = start.clone();
                for _ in 0..4 {
                    piece.rotate(&rotation, &IVec2::ZERO);
                }
                assert_eq!(sorted(piece.blocks), sorted(start.blocks.clone()));
            }
        }
    }

    #[test]
    fn glitch_turns_undo_each_other() {
        for shape in GlitchShape::iter() {
            let start = Rustomino::glitch(shape);
            let mut piece = start.clone();
            piece.rotate(&Rotation::Cw, &IVec2::ZERO);
            piece.rotate(&Rotation::Ccw, &IVec2::ZERO);
            assert_eq!(piece.blocks, start.blocks);
        }
    }

    #[test]
    fn glitch_pieces_stay_in_their_box() {
        for shape in GlitchShape::iter() {
            let mut piece = Rustomino::glitch(shape);
            let size = 0..shape.box_size();
            for _ in 0..4 {
                piece.rotate(&Rotation::Cw, &IVec2::ZERO);
                assert!(piece
                    .blocks
                    .iter()
                    .all(|b| size.contains(&b.x) && size.contains(&b.y)));
            }
        }
    }

    #[test]
    fn glitch_turns_like_a_t_piece() {
        // a 3x3 piece turned about its center lands where the table turned T does
        let mut t = Rustomino::new(RustominoType::T);
        let mut pivot = t.clone();
        pivot.rotation = RustominoRotation::pivot();
        for rotation in [Rotation::Cw, Rotation::Cw, Rotation::Ccw, Rotation::Cw] {
            t.rotate(&rotation, &IVec2::ZERO);
            pivot.rotate(&rotation, &IVec2::ZERO);
            assert_eq!(sorted(pivot.blocks.clone()), sorted(t.blocks.clone()));
        }
    }

    #[test]
    fn four_wide_glitches_turn_like_an_i_piece() {
        // a straight four in the drip's box turns where the table turned I does
        let mut i = Rustomino::new(RustominoType::I);
        let mut pivot = Rustomino::glitch(GlitchShape::Drip);
        pivot.blocks = i.blocks.clone();
        for rotation in [Rotation::Cw, Rotation::Cw, Rotation::Ccw, Rotation::Cw] {
            i.rotate(&rotation, &IVec2::ZERO);
            pivot.rotate(&rotation, &IVec2::ZERO);
            assert_eq!(sorted(pivot.blocks.clone()), sorted(i.blocks.clone()));
        }
        assert_eq!(pivot.translation, i.translation);
        assert_eq!(
            pivot.wall_kick_tests(&Rotation::Cw),
            i.wall_kick_tests(&Rotation::Cw)
        );
    }

    #[test]
    fn glitch_pieces_keep_their_shape_on_reset() {
        for shape in GlitchShape::iter() {
            let mut piece = Rustomino::glitch(shape);
            piece.rotate(&Rotation::Cw, &IVec2::ZERO);
            let reset = piece.reset();
            assert_eq!(reset.glitch, Some(shape));
            assert_eq!(reset.blocks, shape.blocks());
        }
    }

    #[test]
    fn glitch_pieces_have_four_blocks_and_are_not_standard_shapes() {
        for shape in GlitchShape::iter() {
            let blocks = shape.blocks();
            assert_eq!(blocks.len(), 4, "{shape:?}");
            // flood fill along edges: a normal piece reaches all four blocks
            let mut reached = vec![blocks[0]];
            let mut i = 0;
            while i < reached.len() {
                let b = reached[i];
                for o in &blocks {
                    if (o.x - b.x).abs() + (o.y - b.y).abs() == 1 && !reached.contains(o) {
                        reached.push(*o);
                    }
                }
                i += 1;
            }
            assert!(reached.len() < 4, "{shape:?} is joined edge to edge");
        }
    }

    #[test]
    fn standard_mode_only_deals_gentle_glitches() {
        let mut bag = RustominoBag::new();
        let shapes: Vec<GlitchShape> = std::iter::repeat_with(|| bag.get_next(100, Mode::Standard))
            .filter_map(|r| r.glitch)
            .take(200)
            .collect();
        assert!(shapes.iter().all(|shape| shape.gentle()));
        for shape in GlitchShape::iter().filter(|shape| shape.gentle()) {
            assert!(shapes.contains(&shape), "{shape:?} never dealt");
        }
    }

    #[test]
    fn hard_mode_deals_every_glitch() {
        let mut bag = RustominoBag::new();
        let shapes: Vec<GlitchShape> = std::iter::repeat_with(|| bag.get_next(100, Mode::Hard))
            .filter_map(|r| r.glitch)
            .take(300)
            .collect();
        for shape in GlitchShape::iter() {
            assert!(shapes.contains(&shape), "{shape:?} never dealt");
        }
    }

    #[test]
    fn glitch_chance_rises_with_level_and_caps() {
        assert!(glitch_chance(1) < glitch_chance(5));
        assert_eq!(glitch_chance(100), GLITCH_CHANCE_MAX);
    }

    #[test]
    fn glitches_never_come_twice_in_a_row() {
        let mut bag = RustominoBag::new();
        let mut last = false;
        // max chance so glitches turn up often
        for _ in 0..2000 {
            let glitch = bag.get_next(100, Mode::Hard).glitch.is_some();
            assert!(!(glitch && last));
            last = glitch;
        }
    }

    #[test]
    fn the_bag_still_deals_all_seven_between_glitches() {
        let mut bag = RustominoBag::new();
        let dealt: Vec<RustominoType> = std::iter::repeat_with(|| bag.get_next(100, Mode::Hard))
            .filter(|r| r.glitch.is_none())
            .take(70)
            .map(|r| r.rtype)
            .collect();
        for set in dealt.chunks(7) {
            for rtype in RustominoType::iter() {
                assert!(set.contains(&rtype));
            }
        }
    }
}
